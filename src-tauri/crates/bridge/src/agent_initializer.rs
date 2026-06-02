//! `bridge/agent_initializer.py` — workspace, tools, memory, system prompt.

use std::path::PathBuf;
use std::sync::Arc;

use agent::{
    build_agent_system_prompt, create_memory_manager, restore_agent_messages, Agent, BotLlmModel,
    LlmBridgeConfig, McpToolLoader, ToolManagerConfig,
};
use models::{create_bot, ModelsConfig};
use tracing::info;

use crate::bot_router::resolve_bot_type;
use crate::config_sync::{load_dotenv_into_process, sync_config_to_dotenv_logged};

pub struct AgentInitOptions {
    pub workspace: PathBuf,
    pub config: Arc<ModelsConfig>,
    pub mcp_loader: Arc<McpToolLoader>,
    pub session_id: Option<String>,
    pub channel_type: String,
}

/// Mirrors Python `AgentInitializer`.
pub struct AgentInitializer;

impl AgentInitializer {
    pub fn initialize(opts: AgentInitOptions) -> Result<Agent, String> {
        sync_config_to_dotenv_logged(&opts.config);
        load_dotenv_into_process();

        let enable_knowledge = opts.config.knowledge.unwrap_or(true);
        let memory_manager =
            create_memory_manager(opts.workspace.clone(), &opts.config, enable_knowledge)?;

        let config_arc = opts.config.clone();
        let bot_type = resolve_bot_type(&opts.config)?;
        let bot = create_bot(bot_type, config_arc.clone())?;
        let model_name = opts.config.model_or("deepseek-chat");

        let bridge_cfg = LlmBridgeConfig {
            model: model_name.clone(),
            enable_thinking: opts.config.enable_thinking(),
            reasoning_effort: opts.config.reasoning_effort.clone(),
            channel_type: opts.channel_type.clone(),
            session_id: opts.session_id.clone(),
        };

        let model = Arc::new(BotLlmModel::new(bot, bridge_cfg.clone()));

        let max_steps = opts.config.agent_max_steps.unwrap_or(20);
        let max_context_tokens = opts.config.agent_max_context_tokens;

        let mut agent = Agent::with_tool_config(
            "You are SupportFlow, a helpful desktop assistant.",
            model,
            bridge_cfg,
            ToolManagerConfig {
                workspace_dir: Some(opts.workspace.clone()),
                memory_manager: Some(memory_manager),
                enable_knowledge,
                models_config: Some(config_arc),
                ..Default::default()
            },
        );

        agent.max_steps = max_steps;
        agent.max_context_tokens = max_context_tokens;
        agent.workspace_dir = Some(opts.workspace.clone());
        agent.session_id = opts.session_id.clone();
        agent.mcp_registry = Some(opts.mcp_loader.registry.clone());
        agent.mcp_loader = Some(opts.mcp_loader.clone());

        let full_prompt = build_agent_system_prompt(
            &opts.workspace,
            &agent.tools,
            agent
                .skill_manager
                .lock()
                .expect("skill_manager")
                .as_ref(),
            None,
            true,
            enable_knowledge,
            agent.model.as_ref().map(|m| m.model_name()),
        );
        if !full_prompt.trim().is_empty() {
            agent.system_prompt = full_prompt;
        }

        if let Some(ref sid) = opts.session_id {
            restore_agent_messages(&agent, sid, &opts.workspace, &opts.config);
        }

        if opts.session_id.is_none() {
            info!(
                "[AgentInitializer] workspace={} tools={}",
                opts.workspace.display(),
                agent.tools.len()
            );
        }

        Ok(agent)
    }
}
