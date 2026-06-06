//! 运行时初始化与 sidecar 懒加载。

use std::sync::Arc;
use std::time::Duration;

use crate::context::channel::ChannelBridge;
use crate::context::workspace_console;
use crate::services::agent::McpToolLoader;
use tauri::AppHandle;
use tokio::sync::Mutex;

use super::helpers::{
    deferred_autostart_channels, resolve_agent_dirs, should_skip_deferred_channel_autostart,
};
use super::AgentRuntime;
use crate::services::agent::{build_bridge_stack, load_models_config_from_path};

impl AgentRuntime {
    pub fn app_handle(&self) -> AppHandle {
        self.app.clone()
    }

    pub fn initialize(app: &AppHandle) -> Result<Self, String> {
        let (workspace, config_path) = resolve_agent_dirs(app)?;
        let config = load_models_config_from_path(&config_path);

        if let Ok(script_path) = crate::python::resolve_markitdown_script(app) {
            std::env::set_var("MARKITDOWN_SCRIPT", script_path.to_string_lossy().as_ref());
            crate::log_info!("markitdown script resolved: {}", script_path.display());
        } else {
            crate::log_info!(
                "markitdown script not resolvable via resource (will use dev CARGO fallback if present)"
            );
        }

        let mcp_loader = McpToolLoader::new(workspace.clone());
        mcp_loader.ensure_background_load();
        let session_id = format!("session_{}", uuid::Uuid::new_v4());
        let _ = workspace_console::upsert_session_index(&workspace, &session_id, Some("New Chat"));
        let channel_bridge = Arc::new(ChannelBridge::new());
        let _ = channel_bridge.sync_from_config_file(&config_path);
        let bridge_stack = build_bridge_stack(workspace.clone(), &config, mcp_loader.clone());
        let process_hub =
            crate::context::process_hub::ProcessHub::new(workspace.clone(), &config_path);
        Ok(Self {
            app: app.clone(),
            workspace,
            config_path,
            config: tokio::sync::RwLock::new(config),
            mcp_loader,
            bridge_stack: tokio::sync::RwLock::new(bridge_stack),
            session_id: Mutex::new(session_id),
            log_streaming: tokio::sync::RwLock::new(false),
            process_hub,
            channel_bridge,
        })
    }

    pub async fn start_sidecar_deferred(self: Arc<Self>) {
        const DEFAULT_DELAY: Duration = Duration::from_secs(2);
        tokio::time::sleep(DEFAULT_DELAY).await;
        if self.process_hub.channel_slot().get().await.is_some() {
            return;
        }
        match self
            .process_hub
            .ensure_channel(&self.app, std::sync::Arc::downgrade(&self))
            .await
        {
            Ok(sidecar) => {
                crate::log_info!("Channel sidecar ready (deferred start)");
                let channels = deferred_autostart_channels(&self.config_path).unwrap_or_default();
                if channels.is_empty() {
                    if should_skip_deferred_channel_autostart() {
                        crate::log_info!(
                            "Channel autostart skipped (DEV_CHANNEL manual-connect preset)"
                        );
                    } else {
                        crate::log_info!("Channel sidecar: no external channels configured");
                    }
                } else {
                    for channel in &channels {
                        if let Err(e) = sidecar.channel_start(channel).await {
                            crate::log_warn!(
                                "Channel autostart start failed for {}: {}",
                                channel,
                                e
                            );
                        }
                    }
                    crate::log_info!("Channel sidecar running: {}", channels.join(", "));
                }
            }
            Err(e) => {
                crate::log_warn!("Channel sidecar deferred start failed: {e}");
            }
        }
    }

    pub async fn ensure_channel_sidecar(
        self: &Arc<Self>,
    ) -> Result<Arc<crate::python::ChannelPythonSidecar>, String> {
        self.process_hub
            .ensure_channel(&self.app, std::sync::Arc::downgrade(self))
            .await
    }
}
