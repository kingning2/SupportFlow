//! Agent runtime — incremental port of Python `agent/`.
//!
//! ## Migration status
//!
//! | Step | Module | Status |
//! |------|--------|--------|
//! | 1 | `protocol/{cancel,task,result,models,context}` | done |
//! | 2 | `protocol/message_utils` | done |
//! | 3 | `protocol/agent_stream` + LLM bridge (`call_llm_stream`) | done |
//! | 4 | `run_stream`, trim, tool exec, `tools/base_tool` | done |
//! | 5 | `protocol/agent` + file tools (`read/write/edit/ls/bash`) | done |
//! | 6 | `send`, memory tools (`memory_search`/`memory_get`) | done |
//! | 7 | MCP registry, `PromptBuilder` / skills | done |
//! | 8 | MCP client + loader | done |
//! | 9 | memory DB, optional tools | pending |

pub mod prompt;
pub mod protocol;
pub mod skills;
pub mod tools;

pub use prompt::{build_agent_system_prompt, load_context_files, ContextFile, PromptBuilder};
pub use protocol::{
    compress_turn_to_text_only, drop_orphaned_tool_results_openai, estimate_message_tokens,
    estimate_text_tokens, get_cancel_registry, model_context_window, repair_tool_use_adjacency,
    sanitize_claude_messages, tools_from_schemas, Agent, AgentAction, AgentActionType,
    AgentCancelledError, AgentEvent, AgentEventCallback, AgentOutput, AgentResult,
    AgentStreamExecutor, AgentStreamHost, AgentToolSchema, BotLlmModel, CallLlmError, CancelHandle,
    CancelTokenRegistry, LlmBridgeConfig, LlmBridgeError, LlmChunkStream, LlmModel, LlmRequest,
    ParsedToolCall, RunStreamError, RunStreamOptions, SchemaStubTool, Task, TaskStatus, TaskType,
    TeamContext, ToolExecutionResult, ToolResult,
};
pub use skills::{format_skills_for_prompt, Skill, SkillEntry, SkillManager};
pub use tools::{
    load_builtin_tools, load_mcp_configs, noop_uploader, AgentTool, BashConfig, BashTool, EditTool,
    FileKeywordMemoryManager, LsTool, McpClient, McpDynamicTool, McpServerConfig, McpServerStatus,
    McpTool, McpToolLoader, McpToolMap, McpToolRegistry, MemoryGetTool, MemoryManager,
    MemorySearchHit, MemorySearchTool, ReadTool, SendFileUploader, SendTool, ToolManagerConfig,
    ToolRunResult, ToolStage, TruncationResult, WorkspaceToolConfig, WriteTool, DEFAULT_MAX_BYTES,
    DEFAULT_MAX_LINES,
};
