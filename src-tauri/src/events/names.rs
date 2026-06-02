//! 事件名常量（与 `src/config/window-events.ts` 保持同步）。

/// 主窗口 label（与 `tauri.conf.json` / 前端 `MAIN_WINDOW_LABEL` 一致）
pub const MAIN_WINDOW_LABEL: &str = "main";

// --- Rust → 前端 ---

pub const SESSION_CHANGED: &str = "session/changed";
pub const MODAL_OPENED: &str = "modal/opened";
pub const MODAL_CLOSED: &str = "modal/closed";
pub const MODAL_OPEN_PANEL: &str = "modal/open-panel";

pub const AGENT_STREAM_CHUNK: &str = "agent/stream-chunk";
pub const AGENT_RUN_FINISHED: &str = "agent/run-finished";
pub const AGENT_LOG_STREAM: &str = "agent/log-stream";
pub const CHANNEL_STATUS_CHANGED: &str = "channel/status-changed";

// --- 前端 → Rust ---

pub const FE_LOG: &str = "fe/log";
pub const FE_LOG_REQ: &str = "fe/log-req";
