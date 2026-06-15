//! Rig agent runtime — LLM orchestration and tool loop.

mod hooks;
mod messages;
mod provider;
mod runner;
mod simple_chat;
mod tool_adapter;

pub use provider::{resolve_credentials, ProviderCredentials, ProviderFamily};
pub use runner::{run_rig_stream, RigRunOutput, RigRunParams};
pub use simple_chat::run_simple_chat;
