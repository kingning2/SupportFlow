//! 渠道生命周期事件、企微同步与前端渠道配置 API。

use std::sync::Arc;

use tauri::Manager;

use crate::events::channel_status_changed_all;
use crate::events::payloads::ChannelStatusChangedPayload;

use super::AgentRuntime;

impl AgentRuntime {
    pub fn handle_channel_notification(&self, params: &serde_json::Value) {
        let channel = params
            .get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let phase = params
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if channel.is_empty() || phase.is_empty() {
            return;
        }
        let payload = ChannelStatusChangedPayload {
            channel,
            phase,
            message: params
                .get("message")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            user_id: params
                .get("user_id")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            display_name: params
                .get("display_name")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            wait_seconds: params.get("wait_seconds").and_then(|v| v.as_i64()),
            qr_code_url: params
                .get("qr_code_url")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            qr_image: params
                .get("qr_image")
                .and_then(|v| v.as_str())
                .map(str::to_string),
        };
        if let Some(store) = self
            .app
            .try_state::<crate::context::channel::ChannelStatusStore>()
        {
            let _ = store.apply(&payload);
        }
        if let Err(e) = channel_status_changed_all(&self.app, &payload) {
            crate::log_warn!("channel status emit failed: {e}");
        }
    }

    #[cfg(feature = "channel-wework")]
    pub fn wework_contacts_synced(&self, wework_user_id: &str) -> Result<bool, String> {
        self.app
            .state::<crate::context::channel::wework_accounts::WeworkAccountsStore>()
            .contacts_synced(wework_user_id)
    }

    #[cfg(not(feature = "channel-wework"))]
    pub fn wework_contacts_synced(&self, _wework_user_id: &str) -> Result<bool, String> {
        Err("wework channel is not enabled in this build".to_string())
    }

    #[cfg(feature = "channel-wework")]
    pub fn wework_mark_contacts_synced(
        &self,
        wework_user_id: &str,
        synced_at: i64,
    ) -> Result<(), String> {
        self.app
            .state::<crate::context::channel::wework_accounts::WeworkAccountsStore>()
            .mark_contacts_synced(wework_user_id, synced_at)
    }

    #[cfg(not(feature = "channel-wework"))]
    pub fn wework_mark_contacts_synced(
        &self,
        _wework_user_id: &str,
        _synced_at: i64,
    ) -> Result<(), String> {
        Err("wework channel is not enabled in this build".to_string())
    }

    pub async fn channel_console_api(
        self: &Arc<Self>,
        path: &str,
        method: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        crate::context::channel::dispatch(&self.app, self, path, method, &body).await
    }

    pub async fn request_wework_contacts_sync(
        self: &Arc<Self>,
    ) -> Result<serde_json::Value, String> {
        let sidecar = self.ensure_channel_sidecar().await?;
        let sidecar_task = sidecar.clone();
        tokio::spawn(async move {
            if let Err(e) = sidecar_task.wework_sync_contacts().await {
                crate::log_warn!("wework contacts sync failed: {e}");
            }
        });
        Ok(serde_json::json!({
            "status": "success",
            "accepted": true,
        }))
    }

    pub async fn channel_python_channels_post(
        self: &Arc<Self>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let action = payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let channel = payload
            .get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let config = payload
            .get("config")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();

        let sidecar = self.ensure_channel_sidecar().await?;
        if action == "connect" || action == "save" {
            crate::context::channel::validate_channel_id(&channel)?;
        }
        let result = match action.as_str() {
            "save" => {
                let applied = crate::context::channel::persist_channel_config(
                    &self.config_path,
                    &channel,
                    &config,
                )?;
                self.reload_config_from_disk().await?;
                self.channel_bridge
                    .sync_from_config_file(&self.config_path)?;

                let restarted = crate::context::channel::should_restart_channel(&channel, &applied);
                if restarted {
                    let _ = sidecar.channel_restart(&channel).await?;
                }

                crate::context::channel::action_response(
                    self.channel_bridge.active_channels().join(","),
                    restarted,
                    applied,
                )
            }
            "connect" => {
                let (channel_type, applied) =
                    crate::context::channel::connect_channel(&self.config_path, &channel, &config)?;
                self.reload_config_from_disk().await?;
                self.channel_bridge
                    .sync_from_config_file(&self.config_path)?;
                let _ = sidecar.channel_start(&channel).await?;
                crate::context::channel::action_response(channel_type, true, applied)
            }
            "disconnect" => {
                let channel_type =
                    crate::context::channel::disconnect_channel(&self.config_path, &channel)?;
                self.reload_config_from_disk().await?;
                self.channel_bridge
                    .sync_from_config_file(&self.config_path)?;
                let _ = sidecar.channel_stop(&channel).await?;
                crate::context::channel::action_response(channel_type, true, Vec::new())
            }
            _ => {
                return Err(format!(
                    "{}: {action}",
                    crate::services::channel::error_code::UNKNOWN_ACTION
                ));
            }
        };
        Ok(result)
    }
}
