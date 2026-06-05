//! `bridge/agent_bridge.py` — per-session agents and `agent_reply`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::agent::{
    conversation_store_for_workspace, get_cancel_registry, persist_agent_run, Agent,
    AgentEventCallback, CancelHandle, McpToolLoader, RunStreamOptions,
};
use models::{Context, Reply, ReplyType};
use serde_json::Value;
use tracing::{error, info};

use super::agent_event_handler::{make_event_callback, AgentEventHandler};
use super::agent_initializer::{AgentInitOptions, AgentInitializer};
use super::bridge::Bridge;

/// Integrates super `Agent` with channel/desktop runtime.
pub struct AgentBridge {
    pub bridge: Arc<Bridge>,
    workspace: PathBuf,
    config: Arc<models::ModelsConfig>,
    mcp_loader: Arc<McpToolLoader>,
    agents: std::sync::Mutex<HashMap<String, Arc<Agent>>>,
    default_agent: std::sync::Mutex<Option<Arc<Agent>>>,
}

impl AgentBridge {
    pub fn new(
        bridge: Arc<Bridge>,
        workspace: PathBuf,
        config: Arc<models::ModelsConfig>,
        mcp_loader: Arc<McpToolLoader>,
    ) -> Arc<Self> {
        Arc::new(Self {
            bridge,
            workspace,
            config,
            mcp_loader,
            agents: std::sync::Mutex::new(HashMap::new()),
            default_agent: std::sync::Mutex::new(None),
        })
    }

    pub fn ensure_agent(
        self: &Arc<Self>,
        session_id: Option<&str>,
        channel_type: &str,
    ) -> Result<Arc<Agent>, String> {
        if session_id.is_none() {
            let mut guard = self.default_agent.lock().expect("default_agent");
            if guard.is_none() {
                *guard = Some(Arc::new(self.init_agent(None, channel_type)?));
            }
            return guard
                .clone()
                .ok_or_else(|| "default agent missing".to_string());
        }
        let sid = session_id.unwrap().to_string();
        let mut map = self.agents.lock().expect("agents");
        if !map.contains_key(&sid) {
            let agent = Arc::new(self.init_agent(Some(sid.clone()), channel_type)?);
            map.insert(sid.clone(), agent);
        }
        map.get(&sid)
            .cloned()
            .ok_or_else(|| format!("agent for session {sid} missing"))
    }

    fn init_agent(&self, session_id: Option<String>, channel_type: &str) -> Result<Agent, String> {
        AgentInitializer::initialize(AgentInitOptions {
            workspace: self.workspace.clone(),
            config: self.config.clone(),
            mcp_loader: self.mcp_loader.clone(),
            session_id: session_id.clone(),
            channel_type: channel_type.to_string(),
        })
    }

    pub async fn agent_reply(
        self: &Arc<Self>,
        query: &str,
        context: Option<Context>,
        on_event: Option<AgentEventCallback>,
        clear_history: bool,
    ) -> Reply {
        let session_id = context
            .as_ref()
            .and_then(|c| c.session_id().map(str::to_string));
        let request_id = context
            .as_ref()
            .and_then(|c| c.get("request_id"))
            .map(str::to_string);
        let channel_type = context
            .as_ref()
            .and_then(|c| c.get("channel_type"))
            .unwrap_or("web")
            .to_string();

        let token_key = request_id
            .clone()
            .or_else(|| session_id.clone())
            .unwrap_or_default();
        let cancel: Option<CancelHandle> = if token_key.is_empty() {
            None
        } else {
            Some(get_cancel_registry().register(&token_key, session_id.as_deref()))
        };

        let agent = match self
            .clone()
            .ensure_agent(session_id.as_deref(), &channel_type)
        {
            Ok(a) => a,
            Err(e) => {
                release_cancel(&request_id, &session_id);
                return Reply::error(format!("Failed to initialize super agent: {e}"));
            }
        };

        let handler = Arc::new(std::sync::Mutex::new(AgentEventHandler::new(
            context.clone(),
            on_event.clone(),
        )));
        let wrapped_cb = make_event_callback(handler.clone());

        let run_result = agent
            .run_stream(
                query,
                RunStreamOptions {
                    on_event: Some(wrapped_cb),
                    clear_history,
                    cancel: cancel.clone(),
                    skill_filter: None,
                },
            )
            .await;

        if let Some(key) = request_id.as_ref().or(session_id.as_ref()) {
            get_cancel_registry().unregister(key);
        }

        handler.lock().expect("handler").log_summary();

        let response = match run_result {
            Ok(r) => r,
            Err(e) => {
                error!("[AgentBridge] agent_reply error: {e}");
                if let Some(ref sid) = session_id {
                    if agent.messages.lock().expect("messages").is_empty() {
                        if let Ok(store) = conversation_store_for_workspace(&self.workspace) {
                            let _ = store.clear_session(sid);
                            info!("[AgentBridge] Cleared DB for session after error: {sid}");
                        }
                    }
                }
                return Reply::error(format!("Agent error: {e}"));
            }
        };

        if let Some(ref sid) = session_id {
            let new_messages = agent
                .last_run_new_messages
                .lock()
                .expect("last_run")
                .clone();
            persist_agent_run(
                &self.workspace,
                &self.config,
                sid,
                &channel_type,
                &new_messages,
            );
        }

        if let Some(file_info) = agent.files_to_send.lock().expect("files").first().cloned() {
            return file_reply(file_info, response);
        }

        Reply::text(response)
    }

    pub fn clear_session(&self, session_id: &str) {
        info!("[AgentBridge] Clearing session: {session_id}");
        self.agents.lock().expect("agents").remove(session_id);
        if self.config.conversation_persistence.unwrap_or(true) {
            if let Ok(store) = conversation_store_for_workspace(&self.workspace) {
                let _ = store.clear_session(session_id);
            }
        }
    }

    pub fn clear_all_sessions(&self) {
        let count = self.agents.lock().expect("agents").len();
        info!("[AgentBridge] Clearing all sessions ({count} total)");
        self.agents.lock().expect("agents").clear();
        *self.default_agent.lock().expect("default_agent") = None;
    }

    pub fn refresh_all_skills(&self) -> usize {
        super::config_sync::load_dotenv_into_process();
        let mut count = 0;
        if let Some(agent) = self.default_agent.lock().expect("default_agent").as_ref() {
            agent.refresh_skills();
            count += 1;
        }
        for agent in self.agents.lock().expect("agents").values() {
            agent.refresh_skills();
            count += 1;
        }
        if count > 0 {
            info!("[AgentBridge] Refreshed skills in {count} agent instance(s)");
        }
        count
    }
}

fn release_cancel(request_id: &Option<String>, session_id: &Option<String>) {
    if let Some(key) = request_id.as_ref().or(session_id.as_ref()) {
        let _ = get_cancel_registry().unregister(key);
    }
}

fn file_reply(file_info: Value, text_response: String) -> Reply {
    let file_type = file_info
        .get("file_type")
        .and_then(|v| v.as_str())
        .unwrap_or("file");
    let file_path = file_info.get("path").and_then(|v| v.as_str()).unwrap_or("");
    let file_name = file_info
        .get("file_name")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| {
            std::path::Path::new(file_path)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
        });

    let file_url = format!("file://{file_path}");
    let mut reply = if file_type == "image" {
        Reply::new(ReplyType::ImageUrl, file_url)
    } else {
        Reply::new(ReplyType::File, file_url)
    };
    reply.file_name = file_name;
    if !text_response.is_empty() {
        reply.text_content = Some(text_response);
    }
    reply
}
