//! 长驻 stdio NDJSON RPC 运行时（渠道 sidecar、MCP stdio、未来 Rust sidecar 均可复用）。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::sync::oneshot;

use crate::backend::ProcessBackend;
use crate::io::{spawn_stderr_reader, tokio_stdout_reader_loop};
use crate::launch::LaunchMode;
use crate::local::ProcessLocalState;
use crate::shared::ProcessSharedContext;
use crate::spec::CommandSpec;
use crate::stdin::StdinLineWriter;
use crate::task::ProcessTaskContext;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[async_trait]
pub trait InboundRpcHandler: Send + Sync {
    async fn handle(&self, request: &Value) -> Value;
}

pub struct StdioJsonRpcConfig {
    pub name: &'static str,
    pub log_prefix: &'static str,
    pub rpc_timeout: Duration,
    pub health_check: Option<(&'static str, Value)>,
}

pub struct StdioJsonRpcRuntime {
    name: &'static str,
    log_prefix: &'static str,
    shared: ProcessSharedContext,
    local: ProcessLocalState,
    command: CommandSpec,
    handler: Arc<dyn InboundRpcHandler>,
    rpc_timeout: Duration,
    health_check: Option<(&'static str, Value)>,
}

impl StdioJsonRpcRuntime {
    pub fn new(
        config: StdioJsonRpcConfig,
        shared: ProcessSharedContext,
        command: CommandSpec,
        handler: Arc<dyn InboundRpcHandler>,
    ) -> Arc<Self> {
        Arc::new(Self {
            name: config.name,
            log_prefix: config.log_prefix,
            shared,
            local: ProcessLocalState::new(),
            command,
            handler,
            rpc_timeout: config.rpc_timeout,
            health_check: config.health_check,
        })
    }

    pub fn process_name(&self) -> &'static str {
        self.name
    }

    pub fn shared(&self) -> &ProcessSharedContext {
        &self.shared
    }

    pub async fn is_running(&self) -> bool {
        if self
            .local
            .external_alive
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return true;
        }
        if let Some(child) = self.local.tokio_child.lock().await.as_mut() {
            return child.try_wait().ok().flatten().is_none();
        }
        false
    }

    pub async fn clear_process_state(&self) {
        self.local.clear().await;
    }

    /// 注册外部托管子进程（Tauri shell）的 stdin 写入器。
    pub async fn mark_external_running(self: &Arc<Self>, writer: Arc<dyn StdinLineWriter>) {
        self.local
            .external_alive
            .store(true, std::sync::atomic::Ordering::SeqCst);
        *self.local.external_stdin.lock().await = Some(writer);
    }

    pub async fn mark_external_stopped(&self) {
        self.local
            .external_alive
            .store(false, std::sync::atomic::Ordering::SeqCst);
        *self.local.external_stdin.lock().await = None;
    }

    pub async fn ensure_running(self: &Arc<Self>, mode: LaunchMode) -> Result<(), String> {
        if self.is_running().await {
            return Ok(());
        }
        self.clear_process_state().await;
        let backend = self.spawn_backend(mode).await?;
        self.clone().start_io_loops(backend).await;
        self.run_health_check().await;
        Ok(())
    }

    async fn run_health_check(self: &Arc<Self>) {
        let Some((method, params)) = &self.health_check else {
            return;
        };
        match tokio::time::timeout(
            Duration::from_secs(15),
            self.rpc_inner(method, params.clone()),
        )
        .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                tracing::warn!(
                    target: "process",
                    "[{}] health check {method} failed: {e}",
                    self.name
                );
            }
            Err(_) => {
                tracing::warn!(
                    target: "process",
                    "[{}] health check {method} timed out after 15s",
                    self.name
                );
            }
        }
    }

    async fn spawn_backend(self: &Arc<Self>, mode: LaunchMode) -> Result<ProcessBackend, String> {
        match mode {
            LaunchMode::Command(spec) => self.spawn_command(&spec).await,
        }
    }

    async fn spawn_command(self: &Arc<Self>, spec: &CommandSpec) -> Result<ProcessBackend, String> {
        spec.validate_program()?;
        let mut cmd = spec.into_tokio_command(&self.shared);
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("{}: spawn failed: {e}", spec.name))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| format!("{}: no stdin", spec.name))?;
        Ok(ProcessBackend::Tokio { child, stdin })
    }

    async fn start_io_loops(self: Arc<Self>, backend: ProcessBackend) {
        match backend {
            ProcessBackend::Tokio { mut child, stdin } => {
                let stdout = child.stdout.take().expect("process: no stdout");
                let stderr = child.stderr.take();
                *self.local.stdin.lock().await = Some(stdin);
                *self.local.tokio_child.lock().await = Some(child);

                if let Some(stderr) = stderr {
                    spawn_stderr_reader(self.name, self.log_prefix, stderr);
                }

                let runtime = Arc::clone(&self);
                tokio::spawn(async move {
                    ProcessTaskContext::scope(
                        ProcessTaskContext {
                            process_name: runtime.name,
                        },
                        async move {
                            let mut reader = BufReader::new(stdout);
                            tokio_stdout_reader_loop(runtime, &mut reader).await;
                        },
                    )
                    .await;
                });
            }
        }
    }

    pub async fn write_stdin_line(&self, line: &str) -> Result<(), String> {
        if let Some(writer) = self.local.external_stdin.lock().await.clone() {
            return writer.write_line(line).await;
        }
        let mut stdin_guard = self.local.stdin.lock().await;
        let stdin = stdin_guard
            .as_mut()
            .ok_or_else(|| format!("{}: stdin not available", self.name))?;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("{}: write stdin: {e}", self.name))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("{}: flush stdin: {e}", self.name))?;
        Ok(())
    }

    pub async fn dispatch_stdout_line(self: &Arc<Self>, trimmed: &str) {
        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(
                    target: "process",
                    "[{}] non-json stdout: {trimmed} ({e})",
                    self.name
                );
                return;
            }
        };

        if msg.get("method").is_some() {
            let resp = self.handler.handle(&msg).await;
            if let Ok(resp_line) = serde_json::to_string(&resp) {
                let mut payload = resp_line;
                payload.push('\n');
                if let Err(e) = self.write_stdin_line(&payload).await {
                    tracing::warn!(
                        target: "process",
                        "[{}] failed to write RPC response: {e}",
                        self.name
                    );
                }
            }
            return;
        }

        if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
            if let Some(tx) = self.local.pending.lock().await.remove(&id) {
                let _ = tx.send(msg);
            }
        }
    }

    pub async fn rpc(self: &Arc<Self>, method: &str, params: Value) -> Result<Value, String> {
        if !self.is_running().await {
            return Err(format!("{}: process not running", self.name));
        }
        self.rpc_inner(method, params).await
    }

    async fn rpc_inner(self: &Arc<Self>, method: &str, params: Value) -> Result<Value, String> {
        let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.local.pending.lock().await.insert(id, tx);

        let req = json!({ "id": id, "method": method, "params": params });
        let mut req_line = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        req_line.push('\n');
        self.write_stdin_line(&req_line).await?;

        let resp = tokio::time::timeout(self.rpc_timeout, rx)
            .await
            .map_err(|_| format!("{}: RPC timed out: {method}", self.name))?
            .map_err(|_| format!("{}: closed while waiting for {method}", self.name))?;

        if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
            return Err(err.to_string());
        }
        resp.get("result")
            .cloned()
            .ok_or_else(|| format!("{}: response missing result for {method}", self.name))
    }

    pub fn default_command(&self) -> &CommandSpec {
        &self.command
    }
}
