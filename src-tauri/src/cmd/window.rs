use tauri::AppHandle;

use crate::utils::window as window_util;

#[tauri::command]
pub async fn open_modal_window(
    app: AppHandle,
    path: String,
    title: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    label: Option<String>,
) -> Result<String, String> {
    let result = window_util::open_modal_window(&app, path, title, width, height, label);

    match &result {
        Ok(label) => crate::log_cmd_ok!("cmd.window.open_modal_window", "label={label}"),
        Err(err) => crate::log_cmd_err!("cmd.window.open_modal_window", err),
    }

    result
}

#[tauri::command]
pub async fn close_modal_window(app: AppHandle, label: String) -> Result<(), String> {
    crate::log_cmd_result!(
        "cmd.window.close_modal_window",
        window_util::close_modal_window(&app, &label),
        "label={label}"
    )
}

#[tauri::command]
pub async fn modal_window_ready(app: AppHandle, label: String) -> Result<(), String> {
    crate::log_cmd_result!(
        "cmd.window.modal_window_ready",
        window_util::modal_window_ready(&app, &label),
        "label={label}"
    )
}

#[tauri::command]
pub async fn preload_modal_window(app: AppHandle) -> Result<(), String> {
    crate::log_cmd_result!(
        "cmd.window.preload_modal_window",
        window_util::preload_modal_window(&app)
    )
}
