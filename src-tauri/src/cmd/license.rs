//! Subscription license-related IPC commands.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::context::license_store::{LicenseStatusDto, LicenseStore};

/// Snapshot computed at startup (machine code + license validity).
#[tauri::command]
pub fn license_get_status(store: State<'_, LicenseStore>) -> Result<LicenseStatusDto, String> {
    store.snapshot()
}

/// Apply pasted activation token: verify, save to app data, unlock if valid.
#[tauri::command]
pub async fn license_apply_activation(
    app: AppHandle,
    token: String,
) -> Result<LicenseStatusDto, String> {
    tokio::task::spawn_blocking(move || {
        let store = app.state::<LicenseStore>();
        store.apply_activation_token(&app, &token)
    })
    .await
    .map_err(|e| format!("activation task failed: {e}"))?
}

/// Pick a binary activation key file from native dialog, decode and apply.
#[tauri::command]
pub async fn license_pick_and_apply_activation_key(
    app: AppHandle,
) -> Result<LicenseStatusDto, String> {
    let selected = app
        .dialog()
        .file()
        .add_filter("License Key", &["key"])
        .blocking_pick_file();

    let Some(file_path) = selected else {
        return Err("activation key file not selected".to_string());
    };

    let path = match file_path {
        tauri_plugin_dialog::FilePath::Path(pb) => pb,
        tauri_plugin_dialog::FilePath::Url(url) => url
            .to_file_path()
            .map_err(|_| "selected key file path is not a local file".to_string())?,
    };

    let key_bytes = std::fs::read(&path).map_err(|e| format!("read key file failed: {e}"))?;
    let token = crate::utils::license_key::decode_token_from_key_bytes(&key_bytes)?;

    tokio::task::spawn_blocking(move || {
        let store = app.state::<LicenseStore>();
        store.apply_activation_token(&app, &token)
    })
    .await
    .map_err(|e| format!("activation task failed: {e}"))?
}
