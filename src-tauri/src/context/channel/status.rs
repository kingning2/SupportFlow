//! Cross-Webview channel runtime status snapshot derived from Python sidecar events.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::events::payloads::ChannelStatusChangedPayload;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ChannelRuntimeStatus {
    pub phase: String,
    pub message: Option<String>,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub wait_seconds: Option<i64>,
    pub qr_code_url: Option<String>,
    pub qr_image: Option<String>,
}

pub struct ChannelStatusStore {
    inner: Mutex<HashMap<String, ChannelRuntimeStatus>>,
}

impl Default for ChannelStatusStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl ChannelStatusStore {
    /// Apply one channel status update emitted by the Python sidecar.
    ///
    /// # Arguments
    ///
    /// * `payload` - Channel lifecycle payload received from Python
    ///
    /// # Returns
    ///
    /// * `()` - Status snapshot updated in memory
    pub fn apply(&self, payload: &ChannelStatusChangedPayload) -> Result<(), String> {
        let mut guard = crate::utils::err::lock_mutex(&self.inner)?;
        guard.insert(
            payload.channel.clone(),
            ChannelRuntimeStatus {
                phase: payload.phase.clone(),
                message: payload.message.clone(),
                user_id: payload.user_id.clone(),
                display_name: payload.display_name.clone(),
                wait_seconds: payload.wait_seconds,
                qr_code_url: payload.qr_code_url.clone(),
                qr_image: payload.qr_image.clone(),
            },
        );
        Ok(())
    }

    /// Read one channel runtime status snapshot.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel id such as `wx` or `wework`
    ///
    /// # Returns
    ///
    /// * `Option<ChannelRuntimeStatus>` - Current status snapshot when present
    pub fn get(&self, channel: &str) -> Result<Option<ChannelRuntimeStatus>, String> {
        let guard = crate::utils::err::lock_mutex(&self.inner)?;
        Ok(guard.get(channel).cloned())
    }
}
