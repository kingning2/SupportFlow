//! Runtime configuration for model providers (mirrors `config.json` keys).

use serde::Deserialize;
use std::path::Path;

use crate::config::const_::BotType;

/// `tools.web_search` block in `config.json`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case", default)]
pub struct WebSearchConfig {
    pub strategy: Option<String>,
    pub provider: Option<String>,
    pub bocha_api_key: Option<String>,
    pub zhipu_search_engine: Option<String>,
    pub zhipu_content_size: Option<String>,
}

/// `tools.vision` block in `config.json`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case", default)]
pub struct VisionConfig {
    pub model: Option<String>,
    pub provider: Option<String>,
}

/// `tools.browser` block in `config.json`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case", default)]
pub struct BrowserConfig {
    pub cdp_endpoint: Option<String>,
    /// Path to local Chrome / Edge / Chromium (skips auto-download).
    pub chrome_executable: Option<String>,
    pub persistent: Option<bool>,
    pub user_data_dir: Option<String>,
    /// Default true: headless system browser.
    pub headless: Option<bool>,
    pub idle_timeout_secs: Option<u64>,
    pub snapshot_max_chars: Option<usize>,
}

/// `tools` namespace in `config.json`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case", default)]
pub struct ToolsConfig {
    pub web_search: Option<WebSearchConfig>,
    pub browser: Option<BrowserConfig>,
    pub vision: Option<VisionConfig>,
}

/// Subset of SupportFlow `config.json` used by the models layer.
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
    pub zhipu_ai_api_base: Option<String>,
    pub qianfan_api_key: Option<String>,
    pub qianfan_api_base: Option<String>,
    pub linkai_api_base: Option<String>,
    pub moonshot_api_key: Option<String>,
    pub ark_api_key: Option<String>,
    pub ark_base_url: Option<String>,
    pub dashscope_api_key: Option<String>,
    pub minimax_api_key: Option<String>,
    pub linkai_api_key: Option<String>,
    pub ollama_api_key: Option<String>,
    pub ollama_api_base: Option<String>,
    pub use_linkai: Option<bool>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub request_timeout: Option<u64>,
    pub conversation_max_tokens: Option<u32>,
    pub expires_in_seconds: Option<u64>,
    pub character_desc: Option<String>,
    pub proxy: Option<String>,
    /// When false, HTTP clients ignore system proxy env (mirrors `use_proxy` in config.json).
    pub use_proxy: Option<bool>,
    pub clear_memory_commands: Option<Vec<String>>,
    /// Use super-agent tool loop for channel messages (default true in SupportFlow Agent).
    pub agent: Option<bool>,
    pub enable_thinking: Option<bool>,
    pub reasoning_effort: Option<String>,
    pub agent_max_steps: Option<u32>,
    pub agent_max_context_tokens: Option<u32>,
    pub agent_max_context_turns: Option<u32>,
    pub conversation_persistence: Option<bool>,
    pub voice_to_text: Option<String>,
    pub text_to_voice: Option<String>,
    pub translate: Option<String>,
    /// Enable knowledge directory in memory sync/search (default true).
    pub knowledge: Option<bool>,
    /// Explicit embedding vendor: openai | linkai | dashscope | doubao | zhipu
    pub embedding_provider: Option<String>,
    pub embedding_model: Option<String>,
    pub embedding_dimensions: Option<u32>,
    pub tools: Option<ToolsConfig>,
}

impl ModelsConfig {
    /// Load from a SupportFlow-style `config.json` file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let text =
            crate::io::read_to_string(path.as_ref()).map_err(|e| format!("read config: {e}"))?;
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

    pub fn agent_enabled(&self) -> bool {
        self.agent.unwrap_or(true)
    }

    pub fn enable_thinking(&self) -> bool {
        self.enable_thinking.unwrap_or(false)
    }

    fn key_configured(value: Option<&str>) -> bool {
        let v = value.unwrap_or("").trim();
        !v.is_empty() && v != "YOUR API KEY" && v != "YOUR_API_KEY"
    }

    pub fn has_openai_key(&self) -> bool {
        Self::key_configured(self.open_ai_api_key.as_deref())
    }

    pub fn has_linkai_key(&self) -> bool {
        Self::key_configured(self.linkai_api_key.as_deref())
    }

    pub fn has_dashscope_key(&self) -> bool {
        Self::key_configured(self.dashscope_api_key.as_deref())
    }

    pub fn has_zhipu_key(&self) -> bool {
        Self::key_configured(self.zhipu_ai_api_key.as_deref())
    }
}
