//! Browser tool settings (`tools.browser` in config.json).

use std::path::{Path, PathBuf};

use chromiumoxide::detection::{default_executable, DetectionOptions};
use models::ModelsConfig;

use crate::tools::utils::path::expand_path;

#[derive(Debug, Clone)]
pub struct BrowserSettings {
    pub cdp_endpoint: String,
    pub chrome_executable: Option<PathBuf>,
    pub user_data_dir: PathBuf,
    pub persistent: bool,
    pub headless: bool,
    pub snapshot_max_chars: usize,
}

impl BrowserSettings {
    pub fn from_models(config: &ModelsConfig) -> Self {
        let bc = config
            .tools
            .as_ref()
            .and_then(|t| t.browser.as_ref());

        let cdp = bc
            .and_then(|b| b.cdp_endpoint.clone())
            .unwrap_or_default()
            .trim()
            .to_string();

        let chrome_executable = bc
            .and_then(|b| b.chrome_executable.as_deref())
            .map(expand_path);

        let persistent = bc.and_then(|b| b.persistent).unwrap_or(true);
        let user_data_dir = bc
            .and_then(|b| b.user_data_dir.as_deref())
            .map(expand_path)
            .unwrap_or_else(|| expand_path("~/.supportflow/browser_profile"));

        let headless = bc.and_then(|b| b.headless).unwrap_or(true);

        let snapshot_max_chars = bc
            .and_then(|b| b.snapshot_max_chars)
            .unwrap_or(30_000);

        Self {
            cdp_endpoint: cdp,
            chrome_executable,
            user_data_dir,
            persistent,
            headless,
            snapshot_max_chars,
        }
    }
}

/// Resolve a local browser binary (never downloads Chromium).
pub fn resolve_chrome_executable(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        return Err(format!(
            "tools.browser.chrome_executable not found: {}",
            path.display()
        ));
    }

    if let Ok(env_path) = std::env::var("CHROME") {
        let path = PathBuf::from(env_path.trim());
        if path.is_file() {
            return Ok(path);
        }
    }

    if let Ok(env_path) = std::env::var("CHROME_PATH") {
        let path = PathBuf::from(env_path.trim());
        if path.is_file() {
            return Ok(path);
        }
    }

    if let Some(path) = detect_windows_chrome() {
        return Ok(path);
    }

    default_executable(DetectionOptions::default()).map_err(|e| {
        format!(
            "{e}. Install Google Chrome / Microsoft Edge, set tools.browser.chrome_executable in config.json, \
             or set CHROME env to the browser executable path."
        )
    })
}

#[cfg(windows)]
fn detect_windows_chrome() -> Option<PathBuf> {
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    ];
    for c in candidates {
        let p = PathBuf::from(c);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

#[cfg(not(windows))]
fn detect_windows_chrome() -> Option<PathBuf> {
    None
}
