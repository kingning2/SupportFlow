//! Workspace and config resolution (aligned with `agent_runtime`).

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

pub const ENV_WORKSPACE: &str = "SUPPORT_FLOW_WORKSPACE";
pub const ENV_CONFIG: &str = "CHANNEL_CONFIG_PATH";
pub const ENV_DESKTOP_APP: &str = "SUPPORTFLOW_APP";
pub const PID_FILE: &str = ".supportflow.pid";
pub const LOG_FILE: &str = "supportflow.log";

pub fn default_data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join("SupportFlow"))
        .ok_or_else(|| anyhow!("cannot resolve OS data directory"))
}

pub fn resolve_workspace() -> Result<PathBuf> {
    if let Ok(raw) = env::var(ENV_WORKSPACE) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if p.is_dir() {
                return Ok(p);
            }
            return Err(anyhow!(
                "{ENV_WORKSPACE} is not a directory: {}",
                p.display()
            ));
        }
    }
    let p = default_data_dir()?;
    fs_io::create_dir_all(&p).with_context(|| format!("create workspace {}", p.display()))?;
    Ok(p)
}

/// Bundled / mirrored config path for CLI (no Tauri `AppHandle`).
pub fn resolve_config_path(workspace: &Path) -> Result<PathBuf> {
    if let Ok(raw) = env::var(ENV_CONFIG) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if p.is_file() {
                return Ok(p);
            }
            return Err(anyhow!("{ENV_CONFIG} is not a file: {}", p.display()));
        }
    }

    let mirrored = workspace.join("config.json");
    if mirrored.is_file() {
        return Ok(mirrored);
    }

    // Dev: src-tauri/resources/config.json
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/config.json");
    if dev.is_file() {
        return Ok(dev.canonicalize().unwrap_or(dev));
    }

    let template =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/config-template.json");
    if template.is_file() {
        return Ok(template.canonicalize().unwrap_or(template));
    }

    Err(anyhow!(
        "no config.json found. Set {ENV_CONFIG} or copy config-template.json to src-tauri/resources/config.json"
    ))
}

pub fn skills_dir(workspace: &Path) -> PathBuf {
    workspace.join("skills")
}

pub fn knowledge_dir(workspace: &Path) -> PathBuf {
    workspace.join("knowledge")
}

pub fn pid_path(workspace: &Path) -> PathBuf {
    workspace.join(PID_FILE)
}

pub fn log_path(workspace: &Path) -> PathBuf {
    workspace.join(LOG_FILE)
}

pub fn skills_config_path(workspace: &Path) -> PathBuf {
    skills_dir(workspace).join("skills_config.json")
}
