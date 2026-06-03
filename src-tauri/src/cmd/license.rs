//! Subscription license-related IPC commands.

use tauri::{AppHandle, Manager, State};

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
