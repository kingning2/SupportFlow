//! 跨 Webview 共享应用状态，由 Tauri `.manage(...)` 持有。
pub mod agent_runtime;
pub mod channel;
pub mod license_store;
pub mod metrics;
#[cfg(feature = "desktop")]
pub mod process_hub;
