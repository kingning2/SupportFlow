//! 跨 Webview 共享应用态：由 Tauri `.manage` 持有，经 command 读取、`events` 广播。

pub mod agent_runtime;
pub mod channel_bridge;
pub mod channel_python_sidecar;
pub mod license_store;
pub mod session;
pub mod wework_accounts;
pub mod workspace_console;
