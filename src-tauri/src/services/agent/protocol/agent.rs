//! `agent/protocol/agent.py` — top-level Agent orchestration.

use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use tracing::{info, warn};

use crate::config::ModelsConfig;
use crate::services::agent::context::build_agent_system_prompt;
use crate::services::agent::protocol::{
    AgentAction, AgentActionType, AgentEventCallback, CancelHandle, LlmBridgeConfig,
    RunStreamError, ToolResult as CapturedToolResult,
};
use crate::services::agent::rig::RigRunParams;
use crate::services::agent::skills::SkillManager;
use crate::services::agent::tools::{
    load_builtin_tools, AgentTool, McpToolLoader, McpToolRegistry, ToolManagerConfig,
    ToolRunResult, ToolStage,
};

#[derive(Default)]
/// Options for [`Agent::run_stream`].
pub struct RunStreamOptions<'a> {
    pub on_event: Option<AgentEventCallback>,
    pub clear_history: bool,
    pub cancel: Option<CancelHandle>,
    pub skill_filter: Option<&'a [String]>,
}

/// Top-level agent (`agent/protocol/agent.py::Agent`).
pub struct Agent {
    pub name: String,
    pub system_prompt: String,
    pub description: String,
    pub bridge: LlmBridgeConfig,
    pub tools: Vec<Arc<dyn AgentTool>>,
    pub max_steps: u32,
    pub messages: Mutex<Vec<Value>>,
    pub captured_actions: Mutex<Vec<AgentAction>>,
    pub workspace_dir: Option<std::path::PathBuf>,
    pub session_id: Option<String>,
    pub enable_builtin_tools: bool,
    pub enable_knowledge: bool,
    pub language: String,
    pub skill_manager: Mutex<Option<SkillManager>>,
    pub mcp_registry: Option<Arc<McpToolRegistry>>,
    pub mcp_loader: Option<Arc<McpToolLoader>>,
    pub models_config: Option<Arc<ModelsConfig>>,
    /// Messages added in the last `run_stream` call.
    pub last_run_new_messages: Mutex<Vec<Value>>,
    /// Files queued for channel send (from send tool).
    pub files_to_send: Mutex<Vec<Value>>,
}

impl Agent {
    /// 使用默认工具配置创建 Agent。
    pub fn new(system_prompt: impl Into<String>, bridge: LlmBridgeConfig) -> Self {
        let workspace_dir = std::env::current_dir().ok();
        Self::with_tool_config(
            system_prompt,
            bridge,
            ToolManagerConfig {
                workspace_dir: workspace_dir.clone(),
                ..Default::default()
            },
        )
    }

    /// 构造 Agent，并加载内置工具、Skills 与 MCP loader。
    pub fn with_tool_config(
        system_prompt: impl Into<String>,
        bridge: LlmBridgeConfig,
        tool_config: ToolManagerConfig,
    ) -> Self {
        let workspace_dir = tool_config.workspace_dir.clone();
        let tools = load_builtin_tools(&tool_config);

        let skill_manager = workspace_dir.as_ref().map(|ws| SkillManager::new(ws, None));

        let mcp_loader = workspace_dir
            .as_ref()
            .map(|ws| McpToolLoader::new(ws.clone()));
        if let Some(loader) = &mcp_loader {
            loader.ensure_background_load();
        }
        let mcp_registry = mcp_loader.as_ref().map(|l| l.registry.clone());

        Self {
            name: "Agent".into(),
            system_prompt: system_prompt.into(),
            description: "AI Agent".into(),
            bridge,
            tools,
            max_steps: 100,
            messages: Mutex::new(Vec::new()),
            captured_actions: Mutex::new(Vec::new()),
            workspace_dir,
            session_id: None,
            enable_builtin_tools: true,
            enable_knowledge: true,
            language: "zh".into(),
            skill_manager: Mutex::new(skill_manager),
            mcp_registry,
            mcp_loader,
            models_config: tool_config.models_config.clone(),
            last_run_new_messages: Mutex::new(Vec::new()),
            files_to_send: Mutex::new(Vec::new()),
        }
    }

    pub fn with_mcp_registry(mut self, registry: Arc<McpToolRegistry>) -> Self {
        self.mcp_registry = Some(registry);
        self
    }

    pub fn with_mcp_loader(mut self, loader: Arc<McpToolLoader>) -> Self {
        self.mcp_registry = Some(loader.registry.clone());
        self.mcp_loader = Some(loader);
        self
    }

    pub fn refresh_skills(&self) {
        if let Some(sm) = self.skill_manager.lock().expect("skill_manager").as_mut() {
            sm.refresh_skills();
            info!(count = sm.list_skills().len(), "Refreshed skills");
        }
    }

    pub fn list_skills(&self) -> Vec<crate::services::agent::skills::SkillEntry> {
        self.skill_manager
            .lock()
            .expect("skill_manager")
            .as_ref()
            .map(|sm| sm.list_skills().into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_skill(&self, name: &str) -> Option<crate::services::agent::skills::SkillEntry> {
        self.skill_manager
            .lock()
            .expect("skill_manager")
            .as_ref()
            .and_then(|sm| sm.get_skill(name).cloned())
    }

    pub fn add_tool(&mut self, tool: Arc<dyn AgentTool>) {
        self.tools.push(tool);
    }

    pub fn clear_history(&self) {
        self.messages.lock().expect("messages").clear();
        self.captured_actions.lock().expect("actions").clear();
    }

    /// Rebuild system prompt from workspace files, tools, skills, and runtime info.
    pub fn get_full_system_prompt(&self, skill_filter: Option<&[String]>) -> String {
        let workspace = match &self.workspace_dir {
            Some(dir) => dir.as_path(),
            None => return self.system_prompt.clone(),
        };

        let tools: Vec<Arc<dyn AgentTool>> = self.tools.clone();
        let runtime_model = Some(self.bridge.model.as_str());

        let built = {
            let mut guard = self.skill_manager.lock().expect("skill_manager");
            if let Some(sm) = guard.as_mut() {
                sm.refresh_skills();
            }
            build_agent_system_prompt(
                workspace,
                &tools,
                guard.as_ref(),
                skill_filter,
                true,
                self.enable_knowledge,
                runtime_model,
            )
        };

        if built.trim().is_empty() {
            self.system_prompt.clone()
        } else {
            built
        }
    }

    pub async fn run_stream(
        &self,
        user_message: &str,
        options: RunStreamOptions<'_>,
    ) -> Result<String, RunStreamError> {
        if options.clear_history {
            self.clear_history();
        }

        if let Some(loader) = &self.mcp_loader {
            loader.refresh_if_changed();
        }

        let config = self
            .models_config
            .as_ref()
            .ok_or_else(|| RunStreamError::Other("No models config for agent".into()))?;

        let full_system_prompt = self.get_full_system_prompt(options.skill_filter);

        let (messages_copy, original_length) = {
            let guard = self.messages.lock().expect("messages");
            (guard.clone(), guard.len())
        };

        let tools_map = self
            .tools
            .iter()
            .map(|t| (t.name().to_string(), t.clone()))
            .collect();

        let output = crate::services::agent::rig::run_rig_stream(RigRunParams {
            config: config.as_ref(),
            bridge: &self.bridge,
            system_prompt: full_system_prompt,
            user_message: user_message.to_string(),
            messages: messages_copy,
            tools: tools_map,
            max_steps: self.max_steps,
            on_event: options.on_event,
            cancel: options.cancel,
            mcp_registry: self.mcp_registry.clone(),
        })
        .await?;

        {
            let mut guard = self.messages.lock().expect("messages");
            *guard = output.messages.clone();
            let trim_adjusted_start = original_length.min(output.messages.len());
            *self.last_run_new_messages.lock().expect("last_run") =
                output.messages[trim_adjusted_start..].to_vec();
        }

        *self.files_to_send.lock().expect("files") = output.files_to_send.clone();

        self.execute_post_process_tools().await;

        Ok(output.response)
    }

    async fn execute_post_process_tools(&self) {
        for tool in &self.tools {
            if tool.stage() != ToolStage::PostProcess {
                continue;
            }
            let start = std::time::Instant::now();
            let result: ToolRunResult = tool.execute(json!({})).await;
            let execution_time = start.elapsed().as_secs_f64();
            self.capture_tool_use(
                tool.name(),
                json!({}),
                result.result.clone(),
                &result.status,
                None,
                execution_time,
            );
            if result.status == "success" {
                info!(tool = tool.name(), ?result.result, "Post-process tool");
            } else {
                warn!(tool = tool.name(), ?result.result, "Post-process tool failed");
            }
        }
    }

    pub fn capture_tool_use(
        &self,
        tool_name: &str,
        input_params: Value,
        output: Value,
        status: &str,
        thought: Option<String>,
        execution_time: f64,
    ) -> AgentAction {
        let input_map = match input_params {
            Value::Object(m) => m,
            _ => serde_json::Map::new(),
        };

        let tool_result = CapturedToolResult {
            tool_name: tool_name.to_string(),
            input_params: input_map,
            output,
            status: status.to_string(),
            error_message: if status == "error" {
                Some("tool error".into())
            } else {
                None
            },
            execution_time,
        };

        let mut action = AgentAction::new(
            format!("{:p}", self),
            self.name.clone(),
            AgentActionType::ToolUse,
        );
        action.tool_result = Some(tool_result);
        action.thought = thought;

        self.captured_actions
            .lock()
            .expect("actions")
            .push(action.clone());
        action
    }
}
