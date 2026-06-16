//! Shared channel adapter contract: lifecycle phases, error codes, IPC method names.

use typeshare::typeshare;

/// Sidecar lifecycle phases emitted via `channel.notify` → `channel/status-changed`.
pub mod phase {
    pub const STARTING: &str = "starting";
    pub const WAITING_LOGIN: &str = "waiting_login";
    pub const WAITING_SCAN: &str = "waiting_scan";
    pub const SCANNED: &str = "scanned";
    pub const LOGGED_IN: &str = "logged_in";
    pub const SYNCING: &str = "syncing";
    pub const READY: &str = "ready";
    pub const ERROR: &str = "error";
    pub const STOPPED: &str = "stopped";
}

/// Stable error codes for channel actions and sidecar RPC.
pub mod error_code {
    pub const UNKNOWN_CHANNEL: &str = "channel.unknown";
    pub const UNKNOWN_ACTION: &str = "channel.unknown_action";
    pub const SIDECAR_NOT_RUNNING: &str = "channel.sidecar_not_running";
    pub const CONFIG_INVALID: &str = "channel.config_invalid";
    pub const STARTUP_FAILED: &str = "channel.startup_failed";
    pub const SEND_FAILED: &str = "channel.send_failed";
}

/// Rust → Python sidecar RPC methods (stdio NDJSON).
pub mod sidecar_rpc {
    pub const START: &str = "channel.start";
    pub const STOP: &str = "channel.stop";
    pub const RESTART: &str = "channel.restart";
    pub const PING: &str = "ping";
}

/// Python → Rust sidecar inbound RPC methods.
pub mod inbound_rpc {
    pub const AGENT_REPLY: &str = "agent.reply";
    pub const PROCESS: &str = "channel.process";
    pub const DECORATE_TEXT: &str = "channel.decorate_text";
    pub const EXTRACT_MEDIA: &str = "channel.extract_media";
    pub const NOTIFY: &str = "channel.notify";
    pub const MESSAGE: &str = "channel.message";
}

/// Desktop Tauri events (see `events/names.rs`).
pub mod event {
    pub const STATUS_CHANGED: &str = "channel/status-changed";
}

#[typeshare]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelTypeId {
    Wework,
}

impl ChannelTypeId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wework => "wework",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "wework" => Some(Self::Wework),
            _ => None,
        }
    }
}

#[typeshare]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelAdapterCapability {
    Connect,
    Disconnect,
    ListConversations,
    Send,
    OnMessage,
    Health,
}
