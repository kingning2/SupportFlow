//! Model provider catalog for console UI (read-only; mirrors supported `BotType`s).

use crate::config::const_::BotType;
use crate::config::ModelsConfig;

#[derive(Debug, Clone)]
pub struct ModelProviderDescriptor {
    pub id: String,
    pub configured: bool,
    pub is_active: bool,
}

fn has_key(key: &Option<String>) -> bool {
    key.as_ref().is_some_and(|s| !s.trim().is_empty())
}

fn non_empty_ollama_base(config: &ModelsConfig) -> bool {
    config
        .ollama_api_base
        .as_deref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(true)
}

/// Whether `config.json` has credentials for this vendor.
pub fn provider_configured(bot_type: BotType, config: &ModelsConfig) -> bool {
    match bot_type {
        BotType::OpenAi | BotType::Openai | BotType::ChatGpt | BotType::ChatGptOnAzure => {
            has_key(&config.open_ai_api_key)
        }
        BotType::Deepseek => has_key(&config.deepseek_api_key),
        BotType::ClaudeApi => has_key(&config.claude_api_key),
        BotType::Gemini => has_key(&config.gemini_api_key),
        BotType::ZhipuAi => has_key(&config.zhipu_ai_api_key),
        BotType::Moonshot => has_key(&config.moonshot_api_key),
        BotType::Doubao => has_key(&config.ark_api_key),
        BotType::Qwen | BotType::QwenDashscope => has_key(&config.dashscope_api_key),
        BotType::Minimax => has_key(&config.minimax_api_key),
        BotType::Linkai => has_key(&config.linkai_api_key),
        BotType::Ollama => non_empty_ollama_base(config),
        BotType::Custom => has_key(&config.custom_api_key),
        BotType::Baidu | BotType::Qianfan | BotType::Xunfei | BotType::Modelscope => false,
    }
}

/// Stable display order for the console models page.
const PROVIDER_ORDER: &[BotType] = &[
    BotType::Deepseek,
    BotType::Openai,
    BotType::ChatGptOnAzure,
    BotType::ClaudeApi,
    BotType::Gemini,
    BotType::ZhipuAi,
    BotType::Moonshot,
    BotType::Doubao,
    BotType::QwenDashscope,
    BotType::Minimax,
    BotType::Linkai,
    BotType::Ollama,
    BotType::Custom,
    BotType::Baidu,
    BotType::Qianfan,
    BotType::Xunfei,
    BotType::Modelscope,
];

fn is_active(bot_type: BotType, config: &ModelsConfig) -> bool {
    let active = config.bot_type.trim();
    if active.is_empty() {
        return false;
    }
    if active.eq_ignore_ascii_case(bot_type.as_str()) {
        return true;
    }
    config.bot_type().ok() == Some(bot_type)
}

/// All supported vendors with configured / active flags.
pub fn list_providers(config: &ModelsConfig) -> Vec<ModelProviderDescriptor> {
    PROVIDER_ORDER
        .iter()
        .map(|&bot_type| {
            let id = bot_type.as_str().to_string();
            ModelProviderDescriptor {
                configured: provider_configured(bot_type, config),
                is_active: is_active(bot_type, config),
                id,
            }
        })
        .collect()
}
