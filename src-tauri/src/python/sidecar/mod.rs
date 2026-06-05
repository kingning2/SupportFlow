//! Python 渠道 sidecar（stdio NDJSON RPC）。PyInstaller exe 经 Tauri `externalBin` 拉起，开发态回退 `python -m channel`。

mod handler;
mod spawn;

pub use spawn::spawn_sidecar;

use process_runtime::{ProcessSharedContext, StdioJsonRpcRuntime};
use serde_json::{json, Value};
use std::sync::{Arc, Weak};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::context::agent_runtime::AgentRuntime;
use crate::utils::process_tauri::{apply_env_shell, start_shell_event_loop, ShellStdinWriter};

use handler::ChannelInboundHandler;
use spawn::{
    build_runtime, channel_env_pairs, launch_mode_from_spawn, run_health_check, SpawnMode,
};

pub struct ChannelPythonSidecar {
    app: AppHandle,
    runtime: Arc<StdioJsonRpcRuntime>,
    handler: Arc<ChannelInboundHandler>,
    shell_writer: Arc<ShellStdinWriter>,
    shared: ProcessSharedContext,
}

impl ChannelPythonSidecar {
    fn new(app: AppHandle, shared: ProcessSharedContext) -> Arc<Self> {
        let handler = ChannelInboundHandler::new();
        let runtime = build_runtime(shared.clone(), handler.clone());
        Arc::new(Self {
            app,
            runtime,
            handler,
            shell_writer: ShellStdinWriter::new(),
            shared,
        })
    }

    pub async fn register_runtime(&self, runtime: Weak<AgentRuntime>) {
        self.handler.register_runtime(runtime).await;
    }

    pub async fn ensure_running(self: &Arc<Self>) -> Result<(), String> {
        if self.runtime.is_running().await {
            return Ok(());
        }
        let mode = spawn::resolve_spawn_mode(&self.app)?;
        self.ensure_running_with_mode(mode).await
    }

    pub(crate) async fn ensure_running_with_mode(
        self: &Arc<Self>,
        mode: SpawnMode,
    ) -> Result<(), String> {
        if self.runtime.is_running().await {
            return Ok(());
        }
        self.runtime.clear_process_state().await;

        let mut spawn_mode = mode;
        loop {
            match self.spawn_mode(spawn_mode.clone()).await {
                Ok(()) => break,
                Err(e) => {
                    #[cfg(debug_assertions)]
                    if matches!(spawn_mode, SpawnMode::TauriSidecar) {
                        if let Some(dev) = crate::python::paths::dev_channel_source_dir() {
                            crate::log_warn!(
                                "Tauri sidecar spawn failed ({e}); falling back to python -m channel"
                            );
                            spawn_mode = SpawnMode::PythonSource { dir: dev };
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }

        run_health_check(&self.runtime).await;
        Ok(())
    }

    async fn spawn_mode(self: &Arc<Self>, mode: SpawnMode) -> Result<(), String> {
        match mode {
            SpawnMode::TauriSidecar => self.spawn_tauri_sidecar().await,
            SpawnMode::PythonSource { .. } | SpawnMode::CustomExe { .. } => {
                let launch = launch_mode_from_spawn(&mode, &self.shared)?;
                self.runtime.ensure_running(launch).await
            }
        }
    }

    async fn spawn_tauri_sidecar(self: &Arc<Self>) -> Result<(), String> {
        let cmd = self
            .app
            .shell()
            .sidecar("channel-sidecar")
            .map_err(|e| format!("resolve channel sidecar: {e}"))?;
        let env = channel_env_pairs(&self.shared);
        let cmd = apply_env_shell(cmd, &env);
        let (rx, child) = cmd
            .spawn()
            .map_err(|e| format!("启动通道 sidecar 失败: {e}"))?;

        self.shell_writer.set_child(child).await;
        self.runtime
            .mark_external_running(self.shell_writer.clone())
            .await;
        start_shell_event_loop(self.runtime.clone(), self.shell_writer.clone(), rx);
        Ok(())
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
        self.runtime.rpc(method, params).await
    }
}
