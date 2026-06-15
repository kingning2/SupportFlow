//! Sync API keys from config.json → `~/.supportflow/.env` (`agent_initializer._migrate_config_to_env`).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::config::ModelsConfig;
use tracing::{info, warn};

const ENV_FILE: &str = ".supportflow/.env";

fn supportflow_env_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(ENV_FILE)
}

/// Keys mirrored from Python `AgentInitializer._migrate_config_to_env`.
pub fn sync_config_to_dotenv(config: &ModelsConfig) -> Result<(), String> {
    let env_file = supportflow_env_path();
    let key_mapping = [
        ("open_ai_api_key", "OPENAI_API_KEY"),
        ("open_ai_api_base", "OPENAI_API_BASE"),
        ("gemini_api_key", "GEMINI_API_KEY"),
        ("claude_api_key", "CLAUDE_API_KEY"),
        ("linkai_api_key", "LINKAI_API_KEY"),
    ];

    let mut existing = read_env_file(&env_file)?;
    let mut updated = false;

    for (config_field, env_key) in key_mapping {
        let value = config_field_value(config, config_field);
        let old = existing.get(env_key).cloned();

        if !value.is_empty() {
            if old.as_deref() == Some(value.as_str()) {
                continue;
            }
            existing.insert(env_key.to_string(), value);
            updated = true;
        } else if old.is_some() {
            existing.remove(env_key);
            updated = true;
        }
    }

    if !updated {
        return Ok(());
    }

    if let Some(parent) = env_file.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut body = String::from(
        "# Environment variables for agent\n# Auto-managed - synced from config.json on startup\n\n",
    );
    for (k, v) in &existing {
        body.push_str(&format!("{k}={v}\n"));
    }
    fs::write(&env_file, body).map_err(|e| e.to_string())?;
    info!(
        "[bridge] Synced API keys from config.json to {}",
        env_file.display()
    );
    Ok(())
}

fn config_field_value(config: &ModelsConfig, field: &str) -> String {
    let raw = match field {
        "open_ai_api_key" => config.open_ai_api_key.as_deref(),
        "open_ai_api_base" => config.open_ai_api_base.as_deref(),
        "gemini_api_key" => config.gemini_api_key.as_deref(),
        "claude_api_key" => config.claude_api_key.as_deref(),
        "linkai_api_key" => config.linkai_api_key.as_deref(),
        _ => None,
    };
    raw.unwrap_or("").trim().to_string()
}

fn read_env_file(path: &std::path::Path) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    if !path.is_file() {
        return Ok(map);
    }
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(map)
}

pub fn load_dotenv_into_process() {
    let path = supportflow_env_path();
    if !path.is_file() {
        return;
    }
    let Ok(text) = fs::read_to_string(&path) else {
        return;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            if !key.is_empty() && std::env::var(key).is_err() {
                std::env::set_var(key, v.trim());
            }
        }
    }
}

pub fn sync_config_to_dotenv_logged(config: &ModelsConfig) {
    if let Err(e) = sync_config_to_dotenv(config) {
        warn!("[bridge] config → .env sync failed: {e}");
    }
}
