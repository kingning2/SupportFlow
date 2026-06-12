//! 事件名常量（与前端保持同步）。

pub const MAIN_WINDOW_LABEL: &str = "main";

pub const MODAL_OPENED: &str = "modal/opened";
pub const MODAL_CLOSED: &str = "modal/closed";
pub const MODAL_OPEN_PANEL: &str = "modal/open-panel";

pub const AGENT_STREAM_CHUNK: &str = "agent/stream-chunk";
pub const AGENT_RUN_FINISHED: &str = "agent/run-finished";
pub const AGENT_LOG_STREAM: &str = "agent/log-stream";
pub const CHANNEL_STATUS_CHANGED: &str = "channel/status-changed";

pub const FE_LOG: &str = "fe/log";
pub const FE_LOG_REQ: &str = "fe/log-req";
