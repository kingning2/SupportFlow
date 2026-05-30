//! `agent_stream.py` — streaming executor and LLM bridge.

mod executor;
mod helpers;
mod llm_bridge;
mod run_stream;
mod tools_exec;
mod trim;
mod turns;

pub use executor::{
    tools_from_schemas, AgentEvent, AgentEventCallback, AgentStreamExecutor, AgentStreamHost,
    AgentToolSchema, CallLlmError, ParsedToolCall, SchemaStubTool,
};
pub use helpers::{
    filter_think_tags, is_context_overflow_error, is_message_format_error, is_rate_limit_error,
    is_retryable_llm_error, parse_tool_args, truncate_reasoning_for_storage,
    MAX_STORED_REASONING_CHARS,
};
pub use llm_bridge::{BotLlmModel, LlmBridgeConfig, LlmBridgeError, LlmChunkStream, LlmModel};
pub use run_stream::RunStreamError;
pub use tools_exec::ToolExecutionResult;
pub use turns::{aggressive_trim_for_overflow, compress_turn, identify_complete_turns, Turn};
