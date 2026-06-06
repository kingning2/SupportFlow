//! Agent console service facade for config mutation and runtime refresh inputs.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use models::{
    clear_provider_credentials, find_provider_meta, set_chat_model, update_provider_credentials,
    ModelsConfig,
};

use crate::services::bridge::BridgeRuntime;

use super::McpToolLoader;

/// Build runtime config snapshots and bridge inputs for the desktop agent console.
pub struct AgentConsoleService {
    workspace: PathBuf,
    config_path: PathBuf,
    mcp_loader: Arc<McpToolLoader>,
}

impl AgentConsoleService {
    /// Create one console service facade scoped to the current workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace` - Agent workspace root directory
    /// * `config_path` - Bundled desktop config path
    /// * `mcp_loader` - Shared MCP loader used to rebuild bridge runtime
    ///
    /// # Returns
    ///
    /// * `AgentConsoleService` - Workspace-scoped console service facade
    pub fn new(
        workspace: impl Into<PathBuf>,
        config_path: impl Into<PathBuf>,
        mcp_loader: Arc<McpToolLoader>,
    ) -> Self {
        Self {
            workspace: workspace.into(),
            config_path: config_path.into(),
            mcp_loader,
        }
    }

    /// Load models config from the bundled desktop config path.
    ///
    /// # Returns
    ///
    /// * `ModelsConfig` - Active desktop models config snapshot
    pub fn load_models_config(&self) -> ModelsConfig {
        load_models_config_from_path(&self.config_path)
    }

    /// Reload config from disk, mirror it into the workspace, and rebuild bridge runtime.
    ///
    /// # Returns
    ///
    /// * `(ModelsConfig, Arc<BridgeRuntime>)` - Fresh config plus rebuilt bridge stack
    pub fn reload_runtime_inputs(&self) -> Result<(ModelsConfig, Arc<BridgeRuntime>), String> {
        let fresh = self.load_models_config();
        let mirror = self.workspace.join("config.json");
        fs::copy(&self.config_path, &mirror)
            .map_err(|e| format!("sync config to workspace: {e}"))?;
        let bridge_stack =
            build_bridge_stack(self.workspace.clone(), &fresh, self.mcp_loader.clone());
        Ok((fresh, bridge_stack))
    }

    /// Update one provider's credentials and optional API base.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - Provider identifier from models catalog
    /// * `api_key` - Optional provider API key
    /// * `api_base` - Optional provider API base
    /// * `api_base_set` - Whether frontend explicitly edited the API base field
    ///
    /// # Returns
    ///
    /// * `Option<(ModelsConfig, Arc<BridgeRuntime>)>` - Fresh runtime inputs when config changed
    pub fn update_provider(
        &self,
        provider_id: &str,
        api_key: Option<&str>,
        api_base: Option<&str>,
        api_base_set: bool,
    ) -> Result<Option<(ModelsConfig, Arc<BridgeRuntime>)>, String> {
        let meta = find_provider_meta(provider_id)
            .ok_or_else(|| format!("unknown provider: {provider_id}"))?;
        let changed =
            update_provider_credentials(&self.config_path, meta, api_key, api_base, api_base_set)?;
        if !changed {
            return Ok(None);
        }
        self.reload_runtime_inputs().map(Some)
    }

    /// Clear one provider's credentials and rebuild runtime inputs.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - Provider identifier from models catalog
    ///
    /// # Returns
    ///
    /// * `(ModelsConfig, Arc<BridgeRuntime>)` - Fresh runtime inputs after clearing credentials
    pub fn clear_provider(
        &self,
        provider_id: &str,
    ) -> Result<(ModelsConfig, Arc<BridgeRuntime>), String> {
        let meta = find_provider_meta(provider_id)
            .ok_or_else(|| format!("unknown provider: {provider_id}"))?;
        clear_provider_credentials(&self.config_path, meta)?;
        self.reload_runtime_inputs()
    }

    /// Set the active provider/model pair for chat requests.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - Optional active provider id
    /// * `model` - Optional active model id
    ///
    /// # Returns
    ///
    /// * `Option<(ModelsConfig, Arc<BridgeRuntime>)>` - Fresh runtime inputs when config changed
    pub fn set_active_chat(
        &self,
        provider_id: Option<&str>,
        model: Option<&str>,
    ) -> Result<Option<(ModelsConfig, Arc<BridgeRuntime>)>, String> {
        let changed = set_chat_model(&self.config_path, provider_id, model)?;
        if !changed {
            return Ok(None);
        }
        self.reload_runtime_inputs().map(Some)
    }
}

/// Load one models config snapshot from a json file path.
///
/// # Arguments
///
/// * `path` - Bundled desktop config json path
///
/// # Returns
///
/// * `ModelsConfig` - Parsed config or default desktop fallback
pub fn load_models_config_from_path(path: &Path) -> ModelsConfig {
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

/// Build a fresh bridge runtime from workspace config inputs.
///
/// # Arguments
///
/// * `workspace` - Agent workspace root directory
/// * `config` - Active models config snapshot
/// * `mcp_loader` - Shared MCP loader used by the bridge runtime
///
/// # Returns
///
/// * `Arc<BridgeRuntime>` - Rebuilt bridge runtime stack
pub fn build_bridge_stack(
    workspace: PathBuf,
    config: &ModelsConfig,
    mcp_loader: Arc<McpToolLoader>,
) -> Arc<BridgeRuntime> {
    Arc::new(BridgeRuntime::new(
        workspace,
        Arc::new(config.clone()),
        mcp_loader,
    ))
}
