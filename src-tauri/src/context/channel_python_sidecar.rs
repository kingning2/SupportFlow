//! Channel sidecar: PyInstaller exe or `python -m channel`, bidirectional stdio NDJSON.

use std::collections::HashMap;
#[cfg(channel_sidecar_embedded)]
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};

use crate::context::agent_runtime::AgentRuntime;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub struct ChannelPythonSidecar {
    child: Mutex<Option<Child>>,
    stdin: Mutex<Option<ChildStdin>>,
    pending: Mutex<HashMap<u64, oneshot::Sender<Value>>>,
    runtime: Mutex<Option<Weak<AgentRuntime>>>,
    sidecar_exe: PathBuf,
    config_path: PathBuf,
    dev_source_dir: Option<PathBuf>,
}

impl ChannelPythonSidecar {
    fn new(
        sidecar_exe: PathBuf,
        config_path: PathBuf,
        dev_source_dir: Option<PathBuf>,
    ) -> Arc<Self> {
        Arc::new(Self {
            child: Mutex::new(None),
            stdin: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
            runtime: Mutex::new(None),
            sidecar_exe,
            config_path,
            dev_source_dir,
        })
    }

    pub async fn register_runtime(&self, runtime: Weak<AgentRuntime>) {
        *self.runtime.lock().await = Some(runtime);
    }

    fn apply_sidecar_env(&self, cmd: &mut Command) {
        cmd.env("PYTHONUNBUFFERED", "1")
            .env("PYTHONIOENCODING", "utf-8")
            .env("TAURI_CHANNEL_MODE", "1")
            .env(
                "CHANNEL_CONFIG_PATH",
                self.config_path.to_string_lossy().to_string(),
            );
        if let Some(dev) = crate::utils::env::get("DEV_CHANNEL") {
            let trimmed = dev.trim();
            if !trimmed.is_empty() {
                cmd.env("DEV_CHANNEL", trimmed);
            }
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        crate::utils::platform::python::apply_background_command_flags(cmd);
    }

    pub async fn ensure_running(self: &Arc<Self>) -> Result<(), String> {
        let mut guard = self.child.lock().await;
        if guard.is_some() {
            if guard
                .as_mut()
                .map(|c| c.try_wait())
                .transpose()
                .map_err(|e| e.to_string())?
                .is_none()
            {
                return Ok(());
            }
            *guard = None;
            *self.stdin.lock().await = None;
            self.pending.lock().await.clear();
        }

        let mut child = if self.sidecar_exe.is_file() {
            let mut cmd = Command::new(&self.sidecar_exe);
            self.apply_sidecar_env(&mut cmd);
            cmd.spawn().map_err(|e| {
                format!(
                    "启动通道 sidecar 失败 ({}): {e}",
                    self.sidecar_exe.display()
                )
            })?
        } else if let Some(src) = &self.dev_source_dir {
            let python = crate::utils::platform::python::resolve_python_executable();
            let mut cmd = Command::new(&python);
            cmd.current_dir(src).arg("-m").arg("channel");
            self.apply_sidecar_env(&mut cmd);
            cmd.spawn()
                .map_err(|e| format!("开发模式启动 Python sidecar 失败: {e}"))?
        } else {
            return Err(sidecar_missing_message());
        };

        let stdin = child.stdin.take().ok_or("channel sidecar: no stdin")?;
        let stdout = child.stdout.take().ok_or("channel sidecar: no stdout")?;
        let stderr = child.stderr.take();

        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                while read_line_lossy(&mut reader, &mut line).await.unwrap_or(0) > 0 {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        forward_python_log_line(trimmed);
                    }
                    line.clear();
                }
            });
        }

        *self.stdin.lock().await = Some(stdin);
        *guard = Some(child);

        let sidecar = Arc::clone(self);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            sidecar.stdout_reader_loop(&mut reader).await;
        });

        match tokio::time::timeout(Duration::from_secs(15), self.rpc_inner("ping", json!({}))).await
        {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                crate::log_warn!("Channel sidecar ping failed: {e}");
            }
            Err(_) => {
                crate::log_warn!(
                    "Channel sidecar ping timed out after 15s (process may still be starting)"
                );
            }
        }
        Ok(())
    }

    async fn stdout_reader_loop(
        self: Arc<Self>,
        reader: &mut BufReader<tokio::process::ChildStdout>,
    ) {
        let mut line = String::new();
        loop {
            line.clear();
            let n = match read_line_lossy(reader, &mut line).await {
                Ok(n) => n,
                Err(e) => {
                    crate::log_warn!("[channel_agent] stdout reader ended: {e}");
                    break;
                }
            };
            if n == 0 {
                *self.child.lock().await = None;
                *self.stdin.lock().await = None;
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let msg: Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(e) => {
                    crate::log_debug!("[channel_agent] non-json stdout: {trimmed} ({e})");
                    continue;
                }
            };

            if msg.get("method").is_some() {
                let resp = self.handle_python_request(&msg).await;
                if let Ok(resp_line) = serde_json::to_string(&resp) {
                    let mut stdin_guard = self.stdin.lock().await;
                    if let Some(stdin) = stdin_guard.as_mut() {
                        let _ = stdin.write_all(resp_line.as_bytes()).await;
                        let _ = stdin.write_all(b"\n").await;
                        let _ = stdin.flush().await;
                    }
                }
                continue;
            }

            if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
                if let Some(tx) = self.pending.lock().await.remove(&id) {
                    let _ = tx.send(msg);
                }
            }
        }
    }

    async fn handle_python_request(self: &Arc<Self>, req: &Value) -> Value {
        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(json!({}));

        let result = match method {
            "agent.reply" => match self.runtime_handle().await {
                Ok(rt) => match rt.channel_reply(&params).await {
                    Ok(reply) => json!({
                        "id": id,
                        "result": {
                            "status": "success",
                            "content": reply.get("content").cloned().unwrap_or(json!("")),
                            "reply_type": reply.get("reply_type").cloned(),
                            "text_content": reply.get("text_content").cloned(),
                            "file_name": reply.get("file_name").cloned(),
                        }
                    }),
                    Err(e) => json!({ "id": id, "error": e }),
                },
                Err(e) => json!({ "id": id, "error": e }),
            },
            "channel.process" => match self.runtime_handle().await {
                Ok(rt) => match rt.channel_process(&params).await {
                    Ok(payload) => json!({ "id": id, "result": payload }),
                    Err(e) => json!({ "id": id, "error": e }),
                },
                Err(e) => json!({ "id": id, "error": e }),
            },
            "channel.decorate_text" => match self.runtime_handle().await {
                Ok(rt) => match rt.channel_decorate_text(&params).await {
                    Ok(text) => json!({ "id": id, "result": { "text": text } }),
                    Err(e) => json!({ "id": id, "error": e }),
                },
                Err(e) => json!({ "id": id, "error": e }),
            },
            "channel.extract_media" => match self.runtime_handle().await {
                Ok(rt) => match rt.channel_extract_media(&params).await {
                    Ok(items) => json!({ "id": id, "result": { "items": items } }),
                    Err(e) => json!({ "id": id, "error": e }),
                },
                Err(e) => json!({ "id": id, "error": e }),
            },
            "channel.notify" => {
                if let Ok(rt) = self.runtime_handle().await {
                    rt.handle_channel_notification(&params);
                }
                json!({
                    "id": id,
                    "result": { "status": "success" }
                })
            }
            "wework.contacts_synced" => match self.runtime_handle().await {
                Ok(rt) => {
                    let wework_user_id = params
                        .get("wework_user_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if wework_user_id.is_empty() {
                        json!({ "id": id, "error": "wework_user_id required" })
                    } else {
                        match rt.wework_contacts_synced(wework_user_id) {
                            Ok(value) => json!({ "id": id, "result": { "value": value } }),
                            Err(e) => json!({ "id": id, "error": e }),
                        }
                    }
                }
                Err(e) => json!({ "id": id, "error": e }),
            },
            "wework.mark_contacts_synced" => match self.runtime_handle().await {
                Ok(rt) => {
                    let wework_user_id = params
                        .get("wework_user_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let synced_at = params
                        .get("synced_at")
                        .and_then(|v| v.as_i64())
                        .unwrap_or_default();
                    if wework_user_id.is_empty() || synced_at <= 0 {
                        json!({ "id": id, "error": "wework_user_id and synced_at required" })
                    } else {
                        match rt.wework_mark_contacts_synced(wework_user_id, synced_at) {
                            Ok(()) => json!({ "id": id, "result": { "status": "success" } }),
                            Err(e) => json!({ "id": id, "error": e }),
                        }
                    }
                }
                Err(e) => json!({ "id": id, "error": e }),
            },
            _ => json!({ "id": id, "error": format!("unknown method: {method}") }),
        };
        result
    }

    async fn runtime_handle(&self) -> Result<Arc<AgentRuntime>, String> {
        self.runtime
            .lock()
            .await
            .as_ref()
            .and_then(|runtime| runtime.upgrade())
            .ok_or_else(|| "AgentRuntime not registered".to_string())
    }

    pub async fn channel_start(self: &Arc<Self>, channel: &str) -> Result<Value, String> {
        self.rpc("channel.start", json!({ "channel": channel }))
            .await
    }

    pub async fn channel_stop(self: &Arc<Self>, channel: &str) -> Result<Value, String> {
        self.rpc("channel.stop", json!({ "channel": channel }))
            .await
    }

    pub async fn channel_restart(self: &Arc<Self>, channel: &str) -> Result<Value, String> {
        self.rpc("channel.restart", json!({ "channel": channel }))
            .await
    }

    pub async fn wework_sync_contacts(self: &Arc<Self>) -> Result<Value, String> {
        self.rpc("wework.sync_contacts", json!({})).await
    }

    async fn rpc(self: &Arc<Self>, method: &str, params: Value) -> Result<Value, String> {
        self.ensure_running().await?;
        self.rpc_inner(method, params).await
    }

    async fn rpc_inner(self: &Arc<Self>, method: &str, params: Value) -> Result<Value, String> {
        let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let req = json!({ "id": id, "method": method, "params": params });
        let req_line = serde_json::to_string(&req).map_err(|e| e.to_string())?;

        {
            let mut stdin_guard = self.stdin.lock().await;
            let stdin = stdin_guard
                .as_mut()
                .ok_or("channel sidecar stdin not available")?;
            stdin
                .write_all(req_line.as_bytes())
                .await
                .map_err(|e| format!("channel sidecar write stdin: {e}"))?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|e| format!("channel sidecar write stdin: {e}"))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("channel sidecar flush stdin: {e}"))?;
        }

        let resp = tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| format!("channel sidecar RPC timed out: {method}"))?
            .map_err(|_| "channel sidecar closed while waiting for response".to_string())?;

        if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
            return Err(err.to_string());
        }
        resp.get("result")
            .cloned()
            .ok_or_else(|| "channel sidecar response missing result".to_string())
    }
}

fn forward_python_log_line(trimmed: &str) {
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        if v.get("type").and_then(|t| t.as_str()) == Some("log") {
            let level = v.get("level").and_then(|l| l.as_str()).unwrap_or("info");
            let msg = v.get("msg").and_then(|m| m.as_str()).unwrap_or(trimmed);
            match level {
                "error" => crate::log_error!("[channel] {msg}"),
                "warning" | "warn" => crate::log_warn!("[channel] {msg}"),
                "debug" => crate::log_debug!("[channel] {msg}"),
                _ => crate::log_info!("[channel] {msg}"),
            }
            return;
        }
    }
    crate::log_debug!("[channel_agent] {trimmed}");
}

fn sidecar_missing_message() -> String {
    format!(
        "未找到通道 sidecar 可执行文件。请先运行: pnpm run build:channel-sidecar\n\
         产物路径: src-tauri/binaries/channel-sidecar-{}{}",
        env!("BUILD_TARGET"),
        std::env::consts::EXE_SUFFIX
    )
}

#[cfg(channel_sidecar_embedded)]
const EMBEDDED_CHANNEL_SIDECAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/binaries/channel-sidecar-",
    env!("BUILD_TARGET"),
    ".exe"
));

/// Release: sidecar PyInstaller exe is linked into `tauri-app` at compile time (`include_bytes`).
/// First run extracts it to app cache (no separate channel_agent-channels exe beside the installer).
#[cfg(channel_sidecar_embedded)]
fn materialize_embedded_sidecar(app: &AppHandle) -> Result<PathBuf, String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("app_cache_dir: {e}"))?;
    fs::create_dir_all(&cache_dir).map_err(|e| format!("create cache dir: {e}"))?;

    let exe_name = format!("channel_agent-channels{}", std::env::consts::EXE_SUFFIX);
    let path = cache_dir.join(&exe_name);
    let len = EMBEDDED_CHANNEL_SIDECAR.len() as u64;
    let up_to_date = path
        .is_file()
        .then(|| fs::metadata(&path).ok().map(|m| m.len() == len))
        .flatten()
        .unwrap_or(false);

    if !up_to_date {
        fs::write(&path, EMBEDDED_CHANNEL_SIDECAR)
            .map_err(|e| format!("write embedded sidecar to {}: {e}", path.display()))?;
        crate::log_info!(
            "Extracted embedded channel sidecar ({} bytes) -> {}",
            len,
            path.display()
        );
    }

    Ok(path)
}

fn resolve_sidecar_paths(_app: &AppHandle) -> (PathBuf, Option<PathBuf>) {
    // Dev: prefer `python -m channel` so stale PyInstaller binaries do not pollute stdout.
    #[cfg(debug_assertions)]
    if crate::utils::env::get("CHANNEL_SIDECAR_EXE").is_none() {
        if let Some(dev) = dev_source_dir() {
            return (PathBuf::new(), Some(dev));
        }
    }

    if let Some(raw) = crate::utils::env::get("CHANNEL_SIDECAR_EXE") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.is_file() {
                return (path, None);
            }
        }
    }

    #[cfg(all(channel_sidecar_embedded, not(debug_assertions)))]
    if let Ok(path) = materialize_embedded_sidecar(app) {
        return (path, None);
    }

    let manifest = crate::utils::path::crate_path("");
    let binaries_name = format!(
        "channel-sidecar-{}{}",
        env!("BUILD_TARGET"),
        std::env::consts::EXE_SUFFIX
    );
    let in_binaries = manifest.join("binaries").join(&binaries_name);
    if in_binaries.is_file() {
        return (in_binaries, None);
    }

    if let Some(dev) = dev_source_dir() {
        return (PathBuf::new(), Some(dev));
    }

    (PathBuf::new(), None)
}

fn dev_source_dir() -> Option<PathBuf> {
    let src = crate::utils::path::crate_path("channel_agent");
    if src.join("channel").join("__main__.py").is_file() {
        src.canonicalize().ok()
    } else {
        None
    }
}

async fn read_line_lossy<R>(reader: &mut R, line: &mut String) -> std::io::Result<usize>
where
    R: AsyncBufReadExt + Unpin,
{
    let mut buf = Vec::new();
    let n = reader.read_until(b'\n', &mut buf).await?;
    if n == 0 {
        line.clear();
        return Ok(0);
    }
    *line = String::from_utf8_lossy(&buf).into_owned();
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }
    Ok(n)
}

pub async fn spawn_sidecar(
    app: &AppHandle,
    config_path: &Path,
) -> Result<Arc<ChannelPythonSidecar>, String> {
    let (sidecar_exe, dev) = resolve_sidecar_paths(app);

    if !sidecar_exe.is_file() && dev.is_none() {
        return Err(sidecar_missing_message());
    }

    let sidecar = ChannelPythonSidecar::new(sidecar_exe, config_path.to_path_buf(), dev);
    sidecar.ensure_running().await?;
    Ok(sidecar)
}
