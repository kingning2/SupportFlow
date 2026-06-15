//! 从 `ModelsConfig` 解析 rig 提供商凭据。

use crate::config::config::ModelsConfig;
use crate::config::const_::BotType;

/// LLM 提供商族，决定使用哪套 rig client。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFamily {
    /// OpenAI Chat Completions 兼容（DeepSeek / 通义 / 豆包等）
    OpenAiCompat,
    /// Anthropic Messages API
    Anthropic,
    /// Google Gemini API
    Gemini,
    /// Moonshot (Kimi) OpenAI 兼容端点
    Moonshot,
}

/// rig 运行时所需的 API 凭据。
#[derive(Debug, Clone)]
pub struct ProviderCredentials {
    pub family: ProviderFamily,
    pub api_key: String,
    pub api_base: String,
    pub model: String,
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

/// 按 `bot_type` 从桌面配置解析 API key、base URL 与模型名。
pub fn resolve_credentials(
    config: &ModelsConfig,
    bot_type: BotType,
) -> Result<ProviderCredentials, String> {
    let model = config.model_or("deepseek-chat").to_string();
    let openai_key = || non_empty(config.open_ai_api_key.clone()).unwrap_or_default();
    let openai_base = || {
        non_empty(config.open_ai_api_base.clone())
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    };

    let creds = match bot_type {
        BotType::Deepseek => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: non_empty(config.deepseek_api_key.clone())
                .or_else(|| non_empty(config.open_ai_api_key.clone()))
                .unwrap_or_default(),
            api_base: non_empty(config.deepseek_api_base.clone())
                .or_else(|| non_empty(config.open_ai_api_base.clone()))
                .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string()),
            model,
        },
        BotType::ClaudeApi => ProviderCredentials {
            family: ProviderFamily::Anthropic,
            api_key: non_empty(config.claude_api_key.clone()).unwrap_or_default(),
            api_base: non_empty(config.claude_api_base.clone())
                .unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            model,
        },
        BotType::Gemini => ProviderCredentials {
            family: ProviderFamily::Gemini,
            api_key: non_empty(config.gemini_api_key.clone()).unwrap_or_default(),
            api_base: non_empty(config.gemini_api_base.clone())
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string()),
            model,
        },
        BotType::Moonshot => ProviderCredentials {
            family: ProviderFamily::Moonshot,
            api_key: non_empty(config.moonshot_api_key.clone()).unwrap_or_default(),
            api_base: "https://api.moonshot.cn/v1".to_string(),
            model,
        },
        BotType::ZhipuAi => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: non_empty(config.zhipu_ai_api_key.clone()).unwrap_or_default(),
            api_base: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            model,
        },
        BotType::Qwen | BotType::QwenDashscope => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: non_empty(config.dashscope_api_key.clone()).unwrap_or_default(),
            api_base: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model,
        },
        BotType::Doubao => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: non_empty(config.ark_api_key.clone()).unwrap_or_default(),
            api_base: non_empty(config.ark_base_url.clone())
                .unwrap_or_else(|| "https://ark.cn-beijing.volces.com/api/v3".to_string()),
            model,
        },
        BotType::Minimax => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: non_empty(config.minimax_api_key.clone()).unwrap_or_default(),
            api_base: "https://api.minimax.chat/v1".to_string(),
            model,
        },
        BotType::Linkai => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: non_empty(config.linkai_api_key.clone()).unwrap_or_default(),
            api_base: "https://api.link-ai.tech/v1".to_string(),
            model,
        },
        BotType::Custom => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: non_empty(config.custom_api_key.clone()).unwrap_or_default(),
            api_base: non_empty(config.custom_api_base.clone()).unwrap_or_else(openai_base),
            model,
        },
        BotType::Modelscope => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: openai_key(),
            api_base: "https://api-inference.modelscope.cn/v1".to_string(),
            model,
        },
        BotType::OpenAi
        | BotType::Openai
        | BotType::ChatGpt
        | BotType::ChatGptOnAzure
        | BotType::Baidu
        | BotType::Qianfan
        | BotType::Xunfei => ProviderCredentials {
            family: ProviderFamily::OpenAiCompat,
            api_key: openai_key(),
            api_base: openai_base(),
            model,
        },
    };

    if creds.api_key.trim().is_empty() {
        return Err(format!(
            "missing API key for provider {}",
            bot_type.as_str()
        ));
    }

    Ok(creds)
}
