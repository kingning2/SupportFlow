//! `agent/tools/tool_manager.py` — built-in tool registration.

use std::path::PathBuf;
use std::sync::Arc;

use crate::tools::bash::{BashConfig, BashTool};
use crate::tools::edit::EditTool;
use crate::tools::ls::LsTool;
use crate::tools::memory::{
    FileKeywordMemoryManager, MemoryGetTool, MemoryManager, MemorySearchTool,
};
use crate::tools::read::ReadTool;
use crate::tools::send::{noop_uploader, SendFileUploader, SendTool};
use crate::tools::workspace::WorkspaceToolConfig;
use crate::tools::write::WriteTool;
use crate::tools::AgentTool;

#[derive(Clone)]
pub struct ToolManagerConfig {
    pub workspace_dir: Option<PathBuf>,
    pub bash_timeout_secs: u64,
    pub bash_safety_mode: bool,
    /// Override memory backend; defaults to keyword search over `memory/` files.
    pub memory_manager: Option<Arc<dyn MemoryManager>>,
    pub user_id: Option<String>,
    pub enable_knowledge: bool,
    pub send_uploader: Option<Arc<dyn SendFileUploader>>,
}

impl Default for ToolManagerConfig {
    fn default() -> Self {
        Self {
            workspace_dir: None,
            bash_timeout_secs: 30,
            bash_safety_mode: true,
            memory_manager: None,
            user_id: None,
            enable_knowledge: true,
            send_uploader: None,
        }
    }
}

/// Load core built-in tools (mirrors `agent.tools.__init__` core set).
pub fn load_builtin_tools(config: &ToolManagerConfig) -> Vec<Arc<dyn AgentTool>> {
    let cwd = config
        .workspace_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let ws = WorkspaceToolConfig { cwd: cwd.clone() };

    let memory: Arc<dyn MemoryManager> = config.memory_manager.clone().unwrap_or_else(|| {
        Arc::new(FileKeywordMemoryManager::new(
            cwd.clone(),
            config.enable_knowledge,
        ))
    });

    let uploader = config.send_uploader.clone().unwrap_or_else(noop_uploader);

    vec![
        Arc::new(ReadTool::new(ws.clone())),
        Arc::new(WriteTool::new(ws.clone())),
        Arc::new(EditTool::new(ws.clone())),
        Arc::new(BashTool::new(BashConfig {
            cwd: cwd.clone(),
            timeout_secs: config.bash_timeout_secs,
            safety_mode: config.bash_safety_mode,
        })),
        Arc::new(LsTool::new(ws.clone())),
        Arc::new(SendTool::new(ws.clone(), uploader)),
        Arc::new(MemorySearchTool::new(
            memory.clone(),
            config.user_id.clone(),
            config.enable_knowledge,
        )),
        Arc::new(MemoryGetTool::new(memory, config.enable_knowledge)),
    ]
}
