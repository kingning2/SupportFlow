//! Captured tool-use actions during an agent run.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActionType {
    ToolUse,
    Thinking,
    FinalAnswer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub input_params: serde_json::Map<String, Value>,
    pub output: Value,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default)]
    pub execution_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub action_type: AgentActionType,
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<ToolResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
    pub timestamp: f64,
}

impl AgentAction {
    pub fn new(
        agent_id: impl Into<String>,
        agent_name: impl Into<String>,
        action_type: AgentActionType,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.into(),
            agent_name: agent_name.into(),
            action_type,
            content: String::new(),
            tool_result: None,
            thought: None,
            timestamp: now_secs(),
        }
    }
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
