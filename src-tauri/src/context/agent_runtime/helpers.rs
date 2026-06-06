//! 初始化路径解析与跨模块共享的纯函数。

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::agent::McpToolLoader;
use crate::agent::SkillEntry;
use crate::bridge::BridgeRuntime;
use crate::events::payloads::{SkillDetail, SkillItem};
use models::ModelsConfig;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

pub(crate) fn skill_to_item(e: &SkillEntry) -> SkillItem {
    SkillItem {
        name: e.skill.name.clone(),
        description: e.skill.description.clone(),
        enabled: e.enabled,
        source: e.skill.source.clone(),
    }
}

pub(crate) fn skill_to_detail(e: &SkillEntry) -> SkillDetail {
    SkillDetail {
        name: e.skill.name.clone(),
        description: e.skill.description.clone(),
        enabled: e.enabled,
        source: e.skill.source.clone(),
        file_path: e.skill.file_path.clone(),
        base_dir: e.skill.base_dir.clone(),
        disable_model_invocation: e.skill.disable_model_invocation,
    }
}

pub(crate) fn should_skip_deferred_channel_autostart() -> bool {
    crate::utils::env::get("DEV_CHANNEL")
        .map(|v| v.trim() == "wework" || v.trim() == "wx")
        .unwrap_or(false)
}

pub(crate) fn deferred_autostart_channels(config_path: &Path) -> Result<Vec<String>, String> {
    let raw = crate::utils::fs::read_to_string(config_path)?;
    let root: serde_json::Value = crate::utils::json::from_str(&raw)?;
    let configured = crate::utils::channel::parse_desktop_channel_types(root.get("channel_type"));
    if let Some(dev_channel) = crate::utils::env::get("DEV_CHANNEL") {
        let trimmed = dev_channel.trim().to_string();
        if trimmed == "wework" || trimmed == "wx" {
            return Ok(Vec::new());
        }
        if !trimmed.is_empty() {
            return Ok(configured
                .into_iter()
                .filter(|name| name == &trimmed)
                .collect());
        }
    }
    Ok(configured)
}

fn resolve_bundled_config(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        let source_config = crate::utils::path::crate_path("resources/config.json");
        if source_config.is_file() {
            crate::log_info!(
                "agent config: dev source resources/config.json -> {}",
                source_config.display()
            );
            return Ok(source_config);
        }
    }

    for name in ["config.json", "config-template.json"] {
        let path = app
            .path()
            .resolve(format!("resources/{name}"), BaseDirectory::Resource)
            .map_err(|e| e.to_string())?;
        if path.is_file() {
            crate::log_info!("agent config: resources/{name} -> {}", path.display());
            return Ok(path);
        }
    }
    Err(
        "missing bundled config: place config.json in src-tauri/resources/ (see config-template.json)"
            .into(),
    )
}

fn resolve_workspace_dir(app: &AppHandle) -> Result<PathBuf, String> {
    const ENV_KEY: &str = "SUPPORT_FLOW_WORKSPACE";

    if let Some(path) = crate::utils::env::dir_from_env(ENV_KEY) {
        return Ok(path);
    }

    app.path()
        .app_data_dir()
        .map(|p| p.join("SupportFlow"))
        .map_err(|e| e.to_string())
}

pub(crate) fn resolve_agent_dirs(app: &AppHandle) -> Result<(PathBuf, PathBuf), String> {
    let config_path = resolve_bundled_config(app)?;
    let workspace = resolve_workspace_dir(app)?;
    crate::utils::fs::create_dir_all(&workspace)?;

    let mirror = workspace.join("config.json");
    fs::copy(&config_path, &mirror).map_err(|e| format!("sync config to workspace: {e}"))?;

    crate::log_info!(
        "agent workspace: {}, config (resources): {}",
        workspace.display(),
        config_path.display()
    );
    Ok((workspace, config_path))
}

pub(crate) fn load_models_config_from_path(path: &Path) -> ModelsConfig {
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

pub(crate) fn build_bridge_stack(
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
