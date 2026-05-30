mod fe_log;

use tauri::AppHandle;

/// 注册所有「前端 → Rust」事件处理器。
pub fn register_fe_handlers(app: &AppHandle) {
    fe_log::register(app);
}
