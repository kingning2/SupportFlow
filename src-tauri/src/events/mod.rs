//! Tauri 事件总线：集中管理事件名、载荷、Rust 收发。
//!
//! - `names`：与前端 `src/config/window-events.ts` 对齐的常量
//! - `payloads`：序列化载荷（部分 typeshare 导出）
//! - `emit`：Rust → 前端
//! - `listen` / `handlers`：前端 → Rust

mod emit;
mod handlers;
mod listen;
pub mod names;
pub mod payloads;

pub use emit::{
    channel_status_changed_all, modal_closed, modal_opened, session_changed_all, session_changed_to,
};
pub use listen::register;
pub use names::MAIN_WINDOW_LABEL;

/// 在 `Builder::setup` 中调用，注册前端事件监听。
pub fn setup(app: &tauri::AppHandle) {
    register(app);
}
