//! `agent/protocol/models.py` — LLM request types for agent ↔ models bridge.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request for agent LLM calls (`LLMRequest` in Python).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    #[serde(default)]
    pub messages: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    /// System prompt (passed separately to `call_with_tools` / Claude API).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

fn default_temperature() -> f32 {
    0.7
}

impl Default for LlmRequest {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            model: None,
            temperature: default_temperature(),
            max_tokens: None,
            stream: false,
            tools: None,
            system: None,
        }
    }
}

impl LlmRequest {
    pub fn with_messages(messages: Vec<Value>) -> Self {
        Self {
            messages,
            ..Default::default()
        }
    }
}
