//! Shared bridge stack for headless CLI runs.

use std::path::Path;
use std::sync::Arc;

use crate::bridge::{load_dotenv_into_process, sync_config_to_dotenv, BridgeRuntime};
use crate::config::ModelsConfig;
use crate::services::agent::McpToolLoader;
use anyhow::Result;

use crate::cli::paths;

pub struct CliRuntime {
    pub workspace: std::path::PathBuf,
    pub config_path: std::path::PathBuf,
    pub config: Arc<ModelsConfig>,
    pub stack: Arc<BridgeRuntime>,
}

impl CliRuntime {
    pub fn load() -> Result<Self> {
        let workspace = paths::resolve_workspace()?;
        let config_path = paths::resolve_config_path(&workspace)?;
        let config = Arc::new(load_models_config(&config_path));
        load_dotenv_into_process();
        let _ = sync_config_to_dotenv(config.as_ref());
        let mcp_loader = McpToolLoader::new(workspace.clone());
        let stack = Arc::new(BridgeRuntime::new(
            workspace.clone(),
            config.clone(),
            mcp_loader.clone(),
        ));
        Ok(Self {
            workspace,
            config_path,
            config,
            stack,
        })
    }
}

fn load_models_config(path: &Path) -> ModelsConfig {
    if path.is_file() {
        if let Ok(cfg) = ModelsConfig::from_json_file(path) {
            return cfg;
        }
    }
    ModelsConfig {
        bot_type: "deepseek".into(),
        model: Some("deepseek-chat".into()),
        ..Default::default()
    }
}
