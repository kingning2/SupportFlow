//! `agent/protocol/context.py` — team / multi-agent context (minimal port).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::task::Task;

/// Output from one agent in a team run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub agent_name: String,
    pub output: String,
}

impl AgentOutput {
    pub fn new(agent_name: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            agent_name: agent_name.into(),
            output: output.into(),
        }
    }
}

/// Group context for multi-agent workflows (`TeamContext` in Python).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamContext {
    pub name: String,
    pub description: String,
    pub rule: String,
    /// Agent instances are runtime-specific; store opaque handles until agents are ported.
    #[serde(default)]
    pub agents: Vec<Value>,
    #[serde(default)]
    pub user_task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<Task>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_short_name: Option<String>,
    #[serde(default)]
    pub agent_outputs: Vec<AgentOutput>,
    #[serde(default)]
    pub current_steps: u32,
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
}

fn default_max_steps() -> u32 {
    100
}

impl TeamContext {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        rule: impl Into<String>,
        agents: Vec<Value>,
        max_steps: u32,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            rule: rule.into(),
            agents,
            max_steps,
            ..Default::default()
        }
    }
}
