//! 桌面应用业务服务（原 `crates/agent`、`crates/bridge`）。

pub mod agent;
pub mod bridge;
#[cfg(feature = "desktop")]
pub mod channel;
pub mod workflow;
