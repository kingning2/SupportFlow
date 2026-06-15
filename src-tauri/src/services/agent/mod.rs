//! Agent runtime — rig-core orchestration with legacy tool implementations.

pub mod console_service;
pub mod context;
pub mod knowledge;
pub mod memory;
pub mod protocol;
pub mod rig;
pub mod skills;
pub mod tools;
#[cfg(feature = "desktop")]
pub mod workspace;

pub use console_service::{build_bridge_stack, load_models_config_from_path, AgentConsoleService};
pub use context::{
    build_agent_system_prompt, format_skills_for_prompt, load_context_files, ContextFile,
    PromptBuilder,
};
pub use knowledge::IngestBatchResult;
pub use memory::{
    conversation_store_for_workspace, create_memory_manager, persist_agent_run,
    restore_agent_messages, ConversationStore,
};
pub use protocol::{
    get_cancel_registry, Agent, AgentAction, AgentActionType, AgentCancelledError, AgentEvent,
    AgentEventCallback, CancelHandle, CancelTokenRegistry, LlmBridgeConfig, RunStreamError,
    RunStreamOptions, ToolResult,
};
pub use skills::{
    hub_api_base, install_skill_source, load_skills_config, register_skill, save_skills_config,
    skills_config_path, skills_dir, InstallSkillResult, Skill, SkillConfigEntry, SkillEntry,
    SkillManager, SkillsConfigMap,
};
pub use tools::{
    load_builtin_tools, load_mcp_configs, noop_uploader, AgentTool, BashConfig, BashTool,
    BrowserSettings, BrowserTool, EditTool, EnvConfigTool, EnvConfigToolConfig,
    FileKeywordMemoryManager, LsTool, McpClient, McpDynamicTool, McpServerConfig, McpServerStatus,
    McpTool, McpToolLoader, McpToolMap, McpToolRegistry, MemoryGetTool, MemoryManager,
    MemorySearchHit, MemorySearchTool, ReadTool, SendFileUploader, SendTool, ToolManagerConfig,
    ToolRunResult, ToolStage, TruncationResult, VisionTool, WebFetchTool, WebSearchSettings,
    WebSearchTool, WorkspaceToolConfig, WriteTool, DEFAULT_MAX_BYTES, DEFAULT_MAX_LINES,
};
#[cfg(feature = "desktop")]
pub use workspace::AgentWorkspaceService;
