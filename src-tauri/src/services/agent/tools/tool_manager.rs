//! `agent/tools/tool_manager.py` — built-in tool registration.

use std::path::PathBuf;
use std::sync::Arc;

use models::ModelsConfig;

use crate::services::agent::tools::bash::{BashConfig, BashTool};
use crate::services::agent::tools::browser::BrowserTool;
use crate::services::agent::tools::edit::EditTool;
use crate::services::agent::tools::env_config::{EnvConfigTool, EnvConfigToolConfig};
use crate::services::agent::tools::ls::LsTool;
use crate::services::agent::tools::memory::{
    FileKeywordMemoryManager, MemoryGetTool, MemoryManager, MemorySearchTool,
};
use crate::services::agent::tools::read::ReadTool;
use crate::services::agent::tools::send::{noop_uploader, SendFileUploader, SendTool};
use crate::services::agent::tools::vision::VisionTool;
use crate::services::agent::tools::web_fetch::WebFetchTool;
use crate::services::agent::tools::web_search::WebSearchTool;
use crate::services::agent::tools::workspace::WorkspaceToolConfig;
use crate::services::agent::tools::write::WriteTool;
use crate::services::agent::tools::AgentTool;

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
    /// Model/config.json snapshot for optional tools (`web_search`).
    pub models_config: Option<Arc<ModelsConfig>>,
    pub env_config: EnvConfigToolConfig,
    /// Called after env_config set/delete (e.g. refresh skills).
    pub on_env_changed: Option<Arc<dyn Fn() + Send + Sync>>,
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
            models_config: None,
            env_config: EnvConfigToolConfig::default(),
            on_env_changed: None,
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

    let mut tools: Vec<Arc<dyn AgentTool>> = vec![
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
        Arc::new(EnvConfigTool::new(EnvConfigToolConfig {
            env_path: config.env_config.env_path.clone(),
            on_change: config
                .on_env_changed
                .clone()
                .or(config.env_config.on_change.clone()),
        })),
        Arc::new(if let Some(models) = &config.models_config {
            WebFetchTool::with_models_config(cwd.clone(), models)
        } else {
            WebFetchTool::new(cwd.clone())
        }),
    ];

    if let Some(models) = &config.models_config {
        if WebSearchTool::is_available(models) {
            tools.push(Arc::new(WebSearchTool::new(models.clone())));
        }
        tools.push(Arc::new(BrowserTool::new(models.as_ref(), cwd)));
        if VisionTool::is_available(models) {
            tools.push(Arc::new(VisionTool::new(models.clone())));
        }
    }

    tools
}
