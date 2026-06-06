//! In-process agent runtime for Tauri IPC (no Python HTTP).
//!
//! 按职责拆分子模块，见 `context/agent_runtime/`。

mod channel;
mod channel_events;
mod console;
mod helpers;
mod logs;
mod session;
mod setup;
mod stream;
mod workspace;
mod workspace_data;

pub use stream::cancel_request;

use std::path::PathBuf;
use std::sync::Arc;

use crate::context::channel::ChannelBridge;
use crate::services::agent::McpToolLoader;
use crate::services::bridge::BridgeRuntime;
use models::ModelsConfig;
use tokio::sync::Mutex;

/// 桌面端 Agent 运行时：工作区、配置、MCP、渠道 sidecar 与控制台状态。
pub struct AgentRuntime {
    pub(crate) app: tauri::AppHandle,
    pub workspace: PathBuf,
    pub config_path: PathBuf,
    pub(crate) config: tokio::sync::RwLock<ModelsConfig>,
    pub mcp_loader: Arc<McpToolLoader>,
    pub(crate) bridge_stack: tokio::sync::RwLock<Arc<BridgeRuntime>>,
    pub(crate) session_id: Mutex<String>,
    pub(crate) log_streaming: tokio::sync::RwLock<bool>,
    pub(crate) process_hub: crate::context::process_hub::ProcessHub,
    pub(crate) channel_bridge: Arc<ChannelBridge>,
}

#[derive(Clone, Debug)]
pub struct RuntimeMemoryItem {
    pub filename: String,
    pub item_type: String,
    pub size: i32,
    pub updated_at: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeTaskItem {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub next_run_at: Option<String>,
}
