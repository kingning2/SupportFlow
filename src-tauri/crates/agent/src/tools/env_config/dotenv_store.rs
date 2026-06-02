//! Read/write `~/.supportflow/.env` (mirrors Python `EnvConfig` file helpers).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::tools::utils::path::{supportflow_config_dir, supportflow_env_file};

pub fn env_file_path() -> PathBuf {
    supportflow_env_file()
}

pub fn ensure_env_file(path: &Path) -> Result<(), String> {
    let dir = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(supportflow_config_dir);
    fs::create_dir_all(&dir).map_err(|e| format!("create env dir: {e}"))?;
    if !path.is_file() {
        fs::File::create(path).map_err(|e| format!("create env file: {e}"))?;
    }
    Ok(())
}

pub fn mask_value(value: &str) -> String {
    if value.is_empty() || value.len() <= 10 {
        return "***".to_string();
    }
    format!(
        "{}***{}",
        &value[..6.min(value.len())],
        &value[value.len().saturating_sub(4)..]
    )
}

pub fn read_env_file(path: &Path) -> HashMap<String, String> {
    let Ok(content) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    parse_dotenv_content(&content)
}

pub fn parse_dotenv_content(content: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            vars.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    vars
}

pub fn write_env_file(path: &Path, env_vars: &HashMap<String, String>) -> Result<(), String> {
    let mut keys: Vec<_> = env_vars.keys().collect();
    keys.sort();
    let mut body = String::from("# Environment variables for agent skills\n");
    body.push_str("# Auto-managed by env_config tool\n\n");
    for key in keys {
        if let Some(value) = env_vars.get(key) {
            body.push_str(key);
            body.push('=');
            body.push_str(value);
            body.push('\n');
        }
    }
    fs::write(path, body).map_err(|e| format!("write env file: {e}"))
}

/// Apply all keys from the file to the current process environment.
pub fn reload_process_env(path: &Path) {
    for (key, value) in read_env_file(path) {
        std::env::set_var(&key, &value);
    }
}

pub fn delete_process_env(key: &str) {
    std::env::remove_var(key);
}
