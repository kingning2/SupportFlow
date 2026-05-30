//! 前端 → Rust：在应用 setup 阶段注册监听。

use tauri::AppHandle;

use super::handlers;

pub fn register(app: &AppHandle) {
    handlers::register_fe_handlers(app);
}
