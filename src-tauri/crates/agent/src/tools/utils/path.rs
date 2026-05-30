//! Path helpers (`common.utils.expand_path` + tool `_resolve_path`).

use std::path::{Path, PathBuf};

pub fn expand_path(path: &str) -> PathBuf {
    let path = path.trim();
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

pub fn resolve_path(cwd: &Path, path: &str) -> PathBuf {
    let expanded = expand_path(path);
    if expanded.is_absolute() {
        expanded
    } else {
        let joined = cwd.join(&expanded);
        joined.canonicalize().unwrap_or(joined)
    }
}

pub fn cow_env_file() -> PathBuf {
    expand_path("~/.cow/.env")
}

pub fn cow_config_dir() -> PathBuf {
    expand_path("~/.cow")
}

pub fn is_cow_env_file(abs: &Path) -> bool {
    abs.canonicalize()
        .ok()
        .zip(cow_env_file().canonicalize().ok())
        .map(|(a, b)| a == b)
        .unwrap_or(false)
}

pub fn is_cow_config_dir(abs: &Path) -> bool {
    abs.canonicalize()
        .ok()
        .zip(cow_config_dir().canonicalize().ok())
        .map(|(a, b)| a == b)
        .unwrap_or(false)
}
