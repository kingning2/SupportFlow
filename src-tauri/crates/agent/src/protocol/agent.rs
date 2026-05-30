//! `agent/protocol/agent.py` — top-level Agent orchestration.

use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use tracing::{info, warn};

use crate::prompt::build_agent_system_prompt;
use crate::protocol::cancel::CancelHandle;
use crate::protocol::result::{AgentAction, AgentActionType, ToolResult as CapturedToolResult};
use crate::protocol::stream::{
    AgentEventCallback, AgentStreamExecutor, AgentStreamHost, LlmBridgeConfig, LlmModel,
    RunStreamError,
};
use crate::protocol::tokens::{context_reserve_tokens, model_context_window};
use crate::skills::SkillManager;
use crate::tools::{
    load_builtin_tools, AgentTool, McpToolLoader, McpToolRegistry, ToolManagerConfig,
    ToolRunResult, ToolStage,
};

const DEFAULT_MAX_CONTEXT_TURNS: u32 = 20;

/// Host bridge from [`Agent`] into [`AgentStreamExecutor`].
struct AgentHostBridge {
    model_name: String,
    max_context_tokens: Option<u32>,
    context_reserve_tokens: Option<u32>,
    session_id: Option<String>,
    on_clear_session: Option<Arc<dyn Fn() + Send + Sync>>,
    on_memory_flush: Option<Arc<dyn Fn(&[Value], &str) + Send + Sync>>,
}

impl AgentStreamHost for AgentHostBridge {
    fn flush_memory_overflow(&self, messages: &[Value]) {
        if let Some(cb) = &self.on_memory_flush {
            cb(messages, "overflow");
        }
    }

    fn clear_session_db(&self) {
        if let Some(cb) = &self.on_clear_session {
            cb();
        } else if let Some(ref sid) = self.session_id {
            info!(session_id = %sid, "clear_session_db (no store wired yet)");
        }
    }

    fn context_window_tokens(&self) -> u32 {
        model_context_window(&self.model_name)
    }

    fn max_context_tokens(&self) -> Option<u32> {
        self.max_context_tokens.or_else(|| {
            let window = self.context_window_tokens();
            let reserve = self
                .context_reserve_tokens
                .unwrap_or_else(|| context_reserve_tokens(window, None));
            Some(window.saturating_sub(reserve))
        })
    }

    fn memory_flush_on_trim(
        &self,
        discarded_messages: &[Value],
        reason: &str,
        _discarded_turn_count: usize,
    ) {
        if let Some(cb) = &self.on_memory_flush {
            cb(discarded_messages, reason);
        }
    }
}

/// Options for [`Agent::run_stream`].
pub struct RunStreamOptions<'a> {
    pub on_event: Option<AgentEventCallback>,
    pub clear_history: bool,
    pub cancel: Option<CancelHandle>,
    pub skill_filter: Option<&'a [String]>,
}

impl Default for RunStreamOptions<'_> {
    fn default() -> Self {
        Self {
            on_event: None,
            clear_history: false,
            cancel: None,
            skill_filter: None,
        }
    }
}

/// Top-level agent (`agent/protocol/agent.py::Agent`).
pub struct Agent {
    pub name: String,
    pub system_prompt: String,
    pub description: String,
    pub model: Option<Arc<dyn LlmModel>>,
    pub bridge: LlmBridgeConfig,
    pub tools: Vec<Arc<dyn AgentTool>>,
    pub max_steps: u32,
    pub max_context_tokens: Option<u32>,
    pub context_reserve_tokens: Option<u32>,
    pub max_context_turns: u32,
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
    /// Messages added in the last `run_stream` call.
    pub last_run_new_messages: Mutex<Vec<Value>>,
    /// Files queued for channel send (from stream executor).
    pub files_to_send: Mutex<Vec<Value>>,
}

impl Agent {
    pub fn new(
        system_prompt: impl Into<String>,
        model: Arc<dyn LlmModel>,
        bridge: LlmBridgeConfig,
    ) -> Self {
        let workspace_dir = std::env::current_dir().ok();
        let tools = load_builtin_tools(&ToolManagerConfig {
            workspace_dir: workspace_dir.clone(),
            ..Default::default()
        });

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
            model: Some(model),
            bridge,
            tools,
            max_steps: 100,
            max_context_tokens: None,
            context_reserve_tokens: None,
            max_context_turns: DEFAULT_MAX_CONTEXT_TURNS,
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

    pub fn list_skills(&self) -> Vec<crate::skills::SkillEntry> {
        self.skill_manager
            .lock()
            .expect("skill_manager")
            .as_ref()
            .map(|sm| sm.list_skills().into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn with_llm_model(
        system_prompt: impl Into<String>,
        model: Arc<dyn LlmModel>,
        bridge: LlmBridgeConfig,
    ) -> Self {
        let mut agent = Self::new(system_prompt, model, bridge);
        agent.tools.clear();
        agent
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
        let runtime_model = self.model.as_ref().map(|m| m.model_name());

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

        let model = self
            .model
            .as_ref()
            .ok_or_else(|| RunStreamError::Other("No model available for agent".into()))?;

        let full_system_prompt = self.get_full_system_prompt(options.skill_filter);

        let (messages_copy, original_length) = {
            let guard = self.messages.lock().expect("messages");
            (guard.clone(), guard.len())
        };

        let model_name = model.model_name().to_string();
        let host: Arc<dyn AgentStreamHost> = Arc::new(AgentHostBridge {
            model_name,
            max_context_tokens: self.max_context_tokens,
            context_reserve_tokens: self.context_reserve_tokens,
            session_id: self.session_id.clone(),
            on_clear_session: None,
            on_memory_flush: None,
        });

        let tool_arcs: Vec<Arc<dyn AgentTool>> = self.tools.clone();

        let mut executor = AgentStreamExecutor::new(
            model.clone(),
            self.bridge.clone(),
            full_system_prompt,
            tool_arcs,
            self.max_steps,
            options.on_event,
            Some(messages_copy),
            self.max_context_turns,
            options.cancel,
            Some(host),
        );
        executor.mcp_registry = self.mcp_registry.clone();

        let response = match executor.run_stream(user_message).await {
            Ok(r) => r,
            Err(e) => {
                if executor.messages.is_empty() {
                    self.messages.lock().expect("messages").clear();
                    info!("Cleared Agent message history after executor recovery");
                }
                return Err(e);
            }
        };

        {
            let mut guard = self.messages.lock().expect("messages");
            *guard = executor.messages.clone();
            let trim_adjusted_start = original_length.min(executor.messages.len());
            *self.last_run_new_messages.lock().expect("last_run") =
                executor.messages[trim_adjusted_start..].to_vec();
        }

        *self.files_to_send.lock().expect("files") = executor.files_to_send.clone();

        self.execute_post_process_tools().await;

        Ok(response)
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
