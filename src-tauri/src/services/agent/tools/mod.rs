//! `agent/tools/` — tool runtime (mirrors Python `agent/tools/`).

mod base_tool;
pub mod bash;
pub mod browser;
pub mod edit;
pub mod env_config;
mod labels_zh;
pub mod ls;
pub mod mcp;
pub mod memory;
mod profile_get;
mod profile_update;
pub mod read;
pub mod send;
mod tool_manager;
pub mod utils;
pub mod vision;
pub mod web_fetch;
pub mod web_search;
mod workspace;
pub mod write;

pub use base_tool::{AgentTool, ToolRunResult, ToolStage};
pub use bash::{BashConfig, BashTool};
pub use browser::{BrowserSettings, BrowserTool};
pub use edit::EditTool;
pub use env_config::{EnvConfigTool, EnvConfigToolConfig};
pub use labels_zh::{tool_description_zh, tool_label};
pub use ls::LsTool;
pub use mcp::{
    load_mcp_configs, McpClient, McpDynamicTool, McpServerConfig, McpServerStatus, McpTool,
    McpToolLoader, McpToolMap, McpToolRegistry,
};
pub use memory::{
    FileKeywordMemoryManager, MemoryGetTool, MemoryManager, MemorySearchHit, MemorySearchTool,
};
pub use profile_get::ProfileGetTool;
pub use profile_update::ProfileUpdateTool;
pub use read::ReadTool;
pub use send::{noop_uploader, SendFileUploader, SendTool};
pub use tool_manager::{load_builtin_tools, ToolManagerConfig};
pub use utils::{
    format_size, truncate_head, truncate_tail, TruncationResult, DEFAULT_MAX_BYTES,
    DEFAULT_MAX_LINES,
};
pub use vision::VisionTool;
pub use web_fetch::WebFetchTool;
pub use web_search::{WebSearchSettings, WebSearchTool};
pub use workspace::WorkspaceToolConfig;
pub use write::WriteTool;
