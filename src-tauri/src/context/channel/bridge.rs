//! External channel message bridge (replaces SupportFlow Agent Python `bridge`).
//!
//! Python sidecar only manages channel **config**; inbound messages and LLM replies
//! are handled here via the desktop `AgentRuntime`.
//!
//! Channel ids and config schema: [`crate::services::channel::registry`].
//! Lifecycle phases and IPC method names: [`crate::services::channel::contract`].

use std::path::Path;
use std::sync::RwLock;

use serde_json::Value;

use crate::services::channel::{is_known_channel, phase};

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
        let unknown: Vec<_> = list
            .iter()
            .filter(|name| !is_known_channel(name))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            crate::log_warn!(
                "[ChannelBridge] config references unregistered channels: {}",
                unknown.join(", ")
            );
        }
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

    pub fn active_channels(&self) -> Vec<String> {
        self.active.read().map(|g| g.clone()).unwrap_or_default()
    }

    /// Map a sidecar lifecycle phase to frontend login status (catalog contract).
    pub fn login_status_for_phase(phase: &str) -> Option<&'static str> {
        match phase {
            phase::STARTING | phase::WAITING_LOGIN | phase::WAITING_SCAN => Some("waiting_scan"),
            phase::SCANNED => Some("scanned"),
            phase::LOGGED_IN | phase::SYNCING | phase::READY => Some("logged_in"),
            phase::ERROR | phase::STOPPED => Some("unknown"),
            _ => None,
        }
    }

    /// Whether the sidecar phase means the channel is actively handling messages.
    pub fn is_active_phase(phase: &str) -> bool {
        phase == phase::READY
    }
}
