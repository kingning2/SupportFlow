//! 会话 ID、Agent 实例访问与流式事件映射。

use std::sync::Arc;

use crate::events::payloads::AgentStreamChunk;
use crate::services::agent::workspace;
use crate::services::agent::{Agent, AgentEvent};

use super::AgentRuntime;

impl AgentRuntime {
    /// Get current runtime session id.
    pub async fn session_id(&self) -> String {
        self.session_id.lock().await.clone()
    }

    /// Create and switch to a new runtime session id.
    pub async fn new_session(&self) -> String {
        let id = format!("session_{}", uuid::Uuid::new_v4());
        *self.session_id.lock().await = id.clone();
        let _ = workspace::upsert_session_index(&self.workspace, &id, Some("New Chat"));
        self.bridge_stack
            .write()
            .await
            .agent_bridge
            .clear_all_sessions();
        id
    }

    /// Ensure in-process agent instance is initialized for the current session.
    pub async fn ensure_agent(&self) -> Result<Arc<Agent>, String> {
        let session_id = self.session_id().await;
        let stack = self.bridge_stack.read().await.clone();
        let agent = stack.agent_bridge.ensure_agent(Some(&session_id), "web")?;
        Ok(agent)
    }

    /// Clear conversation history of current in-process agent.
    pub async fn clear_context(&self) -> Result<(), String> {
        let session_id = self.session_id().await;
        let agent = self.ensure_agent().await?;
        agent.clear_history();
        if self
            .config
            .read()
            .await
            .conversation_persistence
            .unwrap_or(true)
        {
            let store = crate::services::agent::conversation_store_for_workspace(&self.workspace)?;
            store.clear_context(&session_id)?;
        }
        Ok(())
    }

    /// Execute a readonly closure against initialized agent.
    pub async fn with_agent_read<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&Agent) -> R,
    {
        let agent = self.ensure_agent().await?;
        Ok(f(agent.as_ref()))
    }

    /// Execute a closure against initialized agent (`Agent` is behind `Arc`, use interior mutability on tools if needed).
    pub async fn with_agent_write<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&Agent) -> R,
    {
        let agent = self.ensure_agent().await?;
        Ok(f(agent.as_ref()))
    }

    /// Convert low-level agent stream event into frontend stream chunk payload.
    pub fn map_stream_event(request_id: &str, ev: &AgentEvent) -> Option<AgentStreamChunk> {
        let base = |chunk_type: &str, content: Option<String>| AgentStreamChunk {
            request_id: request_id.to_string(),
            chunk_type: chunk_type.to_string(),
            content,
            tool: None,
            arguments: None,
            status: None,
            result: None,
            execution_time: None,
        };

        match ev.event_type.as_str() {
            "reasoning_update" => ev
                .data
                .get("delta")
                .and_then(|v| v.as_str())
                .map(|d| base("reasoning", Some(d.to_string()))),
            "message_update" => ev
                .data
                .get("delta")
                .and_then(|v| v.as_str())
                .map(|d| base("delta", Some(d.to_string()))),
            "tool_execution_start" => Some(AgentStreamChunk {
                request_id: request_id.to_string(),
                chunk_type: "tool_start".into(),
                content: None,
                tool: ev
                    .data
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                arguments: ev.data.get("arguments").cloned(),
                status: None,
                result: None,
                execution_time: None,
            }),
            "tool_execution_end" => Some(AgentStreamChunk {
                request_id: request_id.to_string(),
                chunk_type: "tool_end".into(),
                content: None,
                tool: ev
                    .data
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                arguments: None,
                status: ev
                    .data
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                result: ev.data.get("result").map(|r| {
                    if let Some(s) = r.as_str() {
                        s.to_string()
                    } else {
                        r.to_string()
                    }
                }),
                execution_time: ev.data.get("execution_time").and_then(|v| v.as_f64()),
            }),
            "message_end" => {
                if ev.data.get("cancelled").and_then(|v| v.as_bool()) == Some(true) {
                    Some(base("cancelled", None))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
