//! Runtime configuration for model providers (mirrors `config.json` keys).

use serde::Deserialize;
use std::path::Path;

use crate::const_::BotType;

/// Subset of CowAgent `config.json` used by the models layer.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case", default)]
pub struct ModelsConfig {
    pub bot_type: String,
    pub model: Option<String>,
    pub open_ai_api_key: Option<String>,
    pub open_ai_api_base: Option<String>,
    pub custom_api_key: Option<String>,
    pub custom_api_base: Option<String>,
    pub deepseek_api_key: Option<String>,
    pub deepseek_api_base: Option<String>,
    pub claude_api_key: Option<String>,
    pub claude_api_base: Option<String>,
    pub gemini_api_key: Option<String>,
    pub gemini_api_base: Option<String>,
    pub zhipu_ai_api_key: Option<String>,
    pub moonshot_api_key: Option<String>,
    pub ark_api_key: Option<String>,
    pub ark_base_url: Option<String>,
    pub dashscope_api_key: Option<String>,
    pub minimax_api_key: Option<String>,
    pub linkai_api_key: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub request_timeout: Option<u64>,
    pub conversation_max_tokens: Option<u32>,
    pub expires_in_seconds: Option<u64>,
    pub character_desc: Option<String>,
    pub proxy: Option<String>,
    pub clear_memory_commands: Option<Vec<String>>,
}

impl ModelsConfig {
    /// Load from a CowAgent-style `config.json` file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(path.as_ref()).map_err(|e| format!("read config: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("parse config: {e}"))
    }

    pub fn bot_type(&self) -> Result<BotType, String> {
        if self.bot_type.is_empty() {
            return Err("bot_type is empty".into());
        }
        self.bot_type.parse()
    }

    pub fn model_or(&self, default: &str) -> String {
        self.model
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(default)
            .to_string()
    }

    pub fn temperature_or(&self, default: f32) -> f32 {
        self.temperature.unwrap_or(default)
    }

    pub fn top_p_or(&self, default: f32) -> f32 {
        self.top_p.unwrap_or(default)
    }

    pub fn frequency_penalty_or(&self, default: f32) -> f32 {
        self.frequency_penalty.unwrap_or(default)
    }

    pub fn presence_penalty_or(&self, default: f32) -> f32 {
        self.presence_penalty.unwrap_or(default)
    }

    pub fn request_timeout_secs(&self) -> u64 {
        self.request_timeout.unwrap_or(600)
    }

    /// `conf().get("clear_memory_commands", ["#清除记忆"])`
    pub fn clear_memory_commands(&self) -> Vec<String> {
        self.clear_memory_commands
            .clone()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| vec!["#清除记忆".to_string()])
    }
}
