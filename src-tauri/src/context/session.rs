//! 跨 Webview 会话：各 Webview 为独立 JS 运行时；状态存于 Rust，经 IPC 广播。

use std::fs;
use std::sync::Mutex;

use directories::ProjectDirs;
use tauri::{AppHandle, Manager};

pub use crate::events::payloads::AppSession;

pub struct SessionStore(pub Mutex<AppSession>);

fn lang_store_path() -> Result<std::path::PathBuf, String> {
    ProjectDirs::from("com", "polymerization", "tauri-app")
        .ok_or_else(|| "cannot resolve project dirs".to_string())
        .map(|p| p.data_local_dir().join("app_lang.txt"))
}

pub fn read_stored_lang() -> String {
    match lang_store_path() {
        Ok(path) => match fs::read_to_string(&path) {
            Ok(s) => {
                let s = s.trim();
                if matches!(s, "cn" | "en") {
                    s.to_string()
                } else {
                    "cn".to_string()
                }
            }
            Err(_) => "cn".to_string(),
        },
        Err(_) => "cn".to_string(),
    }
}

pub fn write_stored_lang(lang: &str) -> Result<(), String> {
    if !matches!(lang, "cn" | "en") {
        return Err(format!("unsupported language: {lang}"));
    }
    let path = lang_store_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, lang).map_err(|e| e.to_string())
}

impl SessionStore {
    pub fn load_from_disk() -> Self {
        let current_language = read_stored_lang();
        Self(Mutex::new(AppSession { current_language }))
    }
}

pub fn get_session(app: &AppHandle) -> Result<AppSession, String> {
    let store = app.state::<SessionStore>();
    let guard = store.0.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}

pub fn set_current_language(app: &AppHandle, language: String) -> Result<(), String> {
    if !matches!(language.as_str(), "cn" | "en") {
        return Err(format!("unsupported language: {language}"));
    }
    write_stored_lang(&language)?;

    let session = {
        let store = app.state::<SessionStore>();
        let mut guard = store.0.lock().map_err(|e| e.to_string())?;
        guard.current_language = language;
        guard.clone()
    };

    broadcast_session(app, &session)
}

pub fn broadcast_session(app: &AppHandle, session: &AppSession) -> Result<(), String> {
    crate::events::session_changed_all(app, session)
}

pub fn push_session_to_webview(app: &AppHandle, webview_label: &str) -> Result<(), String> {
    let session = get_session(app)?;
    crate::events::session_changed_to(app, webview_label, &session)
}
