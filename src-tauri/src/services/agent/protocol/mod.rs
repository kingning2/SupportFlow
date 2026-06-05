//! `agent/protocol/` — core types and cancel registry.

mod agent;
mod cancel;
mod context;
mod message_utils;
mod models;
mod result;
mod stream;
mod task;
mod tokens;

pub use agent::{Agent, RunStreamOptions};
pub use cancel::{get_cancel_registry, AgentCancelledError, CancelHandle, CancelTokenRegistry};
pub use context::{AgentOutput, TeamContext};
pub use message_utils::{
    compress_turn_to_text_only, drop_orphaned_tool_results_openai, repair_tool_use_adjacency,
    sanitize_claude_messages,
};
pub use models::LlmRequest;
pub use result::{AgentAction, AgentActionType, AgentResult, ToolResult};
pub use stream::{
    tools_from_schemas, AgentEvent, AgentEventCallback, AgentStreamExecutor, AgentStreamHost,
    AgentToolSchema, BotLlmModel, CallLlmError, LlmBridgeConfig, LlmBridgeError, LlmChunkStream,
    LlmModel, ParsedToolCall, RunStreamError, SchemaStubTool, ToolExecutionResult,
};
pub use task::{Task, TaskStatus, TaskType};
pub use tokens::{estimate_message_tokens, estimate_text_tokens, model_context_window};
