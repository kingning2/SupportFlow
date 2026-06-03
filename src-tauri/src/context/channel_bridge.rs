//! External channel message bridge (replaces SupportFlow Agent Python `bridge`).
//!
//! Python sidecar only manages channel **config**; inbound messages and LLM replies
//! are handled here via the desktop `AgentRuntime`.

use std::path::Path;
use std::sync::RwLock;

use serde_json::Value;

#[derive(Debug, Default)]
pub struct ChannelBridge {
    active: RwLock<Vec<String>>,
}

impl ChannelBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reload `channel_type` from config after Python connect/disconnect/save.
    pub fn sync_from_config_file(&self, config_path: &Path) -> Result<Vec<String>, String> {
        let raw = crate::utils::fs::read_to_string(config_path)?;
        let cfg: Value = crate::utils::json::from_str(&raw)?;
        let list = crate::utils::channel::parse_desktop_channel_types(cfg.get("channel_type"));
        if let Ok(mut guard) = self.active.write() {
            *guard = list.clone();
        }
        crate::log_info!(
            "[ChannelBridge] active channels: {}",
            if list.is_empty() {
                "(none)".into()
            } else {
                list.join(", ")
            }
        );
        Ok(list)
    }

    #[allow(dead_code)]
    pub fn active_channels(&self) -> Vec<String> {
        self.active.read().map(|g| g.clone()).unwrap_or_default()
    }

    /// Apply sidecar `channels.action` result and refresh active set from disk.
    pub fn on_action_result(
        &self,
        config_path: &Path,
        action: &str,
        channel: &str,
        result: &Value,
    ) -> Result<(), String> {
        if result.get("status").and_then(|v| v.as_str()) != Some("success") {
            return Ok(());
        }
        self.sync_from_config_file(config_path)?;
        crate::log_info!(
            "[ChannelBridge] action={action} channel={channel} (runtime messaging via Rust agent)"
        );
        Ok(())
    }
}
