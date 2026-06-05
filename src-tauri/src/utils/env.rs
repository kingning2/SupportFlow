use std::path::PathBuf;

/// Read environment variable as string.
///
/// Returns `None` when the variable is missing or is not valid unicode.
pub fn get(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Resolve an env var into a directory path.
///
/// Returns `Some(path)` only when the (trimmed) env var value is a directory.
/// Otherwise returns `None` and emits logs describing the reason.
pub fn dir_from_env(key: &str) -> Option<PathBuf> {
    let Some(raw) = get(key) else {
        return None;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = PathBuf::from(trimmed);
    if path.is_dir() {
        crate::log_info!("agent workspace from {key}: {}", path.display());
        Some(path)
    } else {
        crate::log_warn!("{key} is not a directory: {}", path.display());
        None
    }
}
