//! Agent core types, cancel registry, and streaming IPC contract.

mod actions;
mod agent;
mod bridge_config;
mod cancel;
mod errors;
mod events;

pub use actions::{AgentAction, AgentActionType, ToolResult};
pub use agent::{Agent, RunStreamOptions};
pub use bridge_config::LlmBridgeConfig;
pub use cancel::{get_cancel_registry, AgentCancelledError, CancelHandle, CancelTokenRegistry};
pub use errors::RunStreamError;
pub use events::{AgentEvent, AgentEventCallback};
