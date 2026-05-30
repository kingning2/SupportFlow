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
    let label = window_util::open_modal_window(&app, path, title, width, height, label)?;
    crate::log_info!("cmd.window.open_modal_window ok label={label}");
    Ok(label)
}

#[tauri::command]
pub async fn close_modal_window(app: AppHandle, label: String) -> Result<(), String> {
    window_util::close_modal_window(&app, &label)?;
    crate::log_info!("cmd.window.close_modal_window ok label={label}");
    Ok(())
}

#[tauri::command]
pub async fn modal_window_ready(app: AppHandle, label: String) -> Result<(), String> {
    window_util::modal_window_ready(&app, &label)?;
    crate::log_info!("cmd.window.modal_window_ready ok label={label}");
    Ok(())
}

#[tauri::command]
pub async fn preload_modal_window(app: AppHandle) -> Result<(), String> {
    window_util::preload_modal_window(&app)?;
    crate::log_info!("cmd.window.preload_modal_window ok");
    Ok(())
}
