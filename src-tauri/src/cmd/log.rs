#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Error,
    Warn,
}

#[tauri::command]
pub async fn log_fe(event: LogLevel, msg: String) {
    match event {
        LogLevel::Info => crate::log_info!("[fe] {}", msg),
        LogLevel::Error => crate::log_error!("[fe] {}", msg),
        LogLevel::Warn => crate::log_warn!("[fe] {}", msg),
    }
}

#[tauri::command]
pub async fn log_fe_req(event: LogLevel, msg: String) {
    match event {
        LogLevel::Info => crate::log_info!("[fe_req] {}", msg),
        LogLevel::Error => crate::log_error!("[fe_req] {}", msg),
        LogLevel::Warn => crate::log_warn!("[fe_req] {}", msg),
    }
}
