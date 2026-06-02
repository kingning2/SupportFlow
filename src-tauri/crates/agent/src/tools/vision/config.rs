//! Vision provider resolution (`vision.py`).

use std::sync::Arc;

use models::const_::BotType;
use models::{create_bot, BotHandle, ModelsConfig};

const DEFAULT_MODEL: &str = "gpt-4.1-mini";

pub struct VisionProvider {
    pub name: String,
    pub model_override: Option<String>,
    pub backend: VisionBackend,
}

pub enum VisionBackend {
    OpenAi {
        api_key: String,
        api_base: String,
    },
    LinkAi {
        api_key: String,
        api_base: String,
    },
    Bot {
        bot: BotHandle,
    },
}

pub fn user_vision_model(config: &ModelsConfig) -> Option<String> {
    config
        .tools
        .as_ref()
        .and_then(|t| t.vision.as_ref())
        .and_then(|v| v.model.as_ref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn user_vision_provider(config: &ModelsConfig) -> Option<String> {
    config
        .tools
        .as_ref()
        .and_then(|t| t.vision.as_ref())
        .and_then(|v| v.provider.as_ref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn valid_key(key: &str) -> bool {
    !key.trim().is_empty() && key != "YOUR API KEY" && key != "YOUR_API_KEY"
}

fn ensure_v1(api_base: &str) -> String {
    let base = api_base.trim_end_matches('/');
    if base.is_empty() {
        return "https://api.openai.com/v1".into();
    }
    if base.split('/').last().is_some_and(|s| s.starts_with('v')) {
        return base.to_string();
    }
    format!("{base}/v1")
}

fn openai_model_ok(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.starts_with("gpt-")
        || lower.starts_with("o1-")
        || lower.starts_with("o3-")
        || lower.starts_with("o4-")
        || lower.starts_with("chatgpt-")
}

fn infer_provider_from_model(model: &str) -> Option<&'static str> {
    let lower = model.to_lowercase();
    if openai_model_ok(&lower) {
        return Some("OpenAI");
    }
    const PREFIXES: &[(&str, &str)] = &[
        ("doubao-", "Doubao"),
        ("kimi-", "Moonshot"),
        ("moonshot-", "Moonshot"),
        ("qwen", "DashScope"),
        ("claude-", "Claude"),
        ("ernie-", "Qianfan"),
        ("gemini-", "Gemini"),
        ("glm-", "ZhipuAI"),
        ("minimax-", "MiniMax"),
        ("abab", "MiniMax"),
    ];
    for (prefix, name) in PREFIXES {
        if lower.starts_with(prefix) {
            return Some(name);
        }
    }
    None
}

fn build_openai(config: &ModelsConfig, model_override: Option<String>) -> Option<VisionProvider> {
    let api_key = config
        .open_ai_api_key
        .as_deref()
        .filter(|k| valid_key(k))?
        .to_string();
    let api_base = config
        .open_ai_api_base
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(ensure_v1)
        .unwrap_or_else(|| "https://api.openai.com/v1".into());
    let mo = model_override.filter(|m| openai_model_ok(m));
    Some(VisionProvider {
        name: "OpenAI".into(),
        model_override: mo,
        backend: VisionBackend::OpenAi { api_key, api_base },
    })
}

fn build_linkai(config: &ModelsConfig, model_override: Option<String>) -> Option<VisionProvider> {
    let api_key = config
        .linkai_api_key
        .as_deref()
        .filter(|k| valid_key(k))?
        .to_string();
    let api_base = config
        .linkai_api_base
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(ensure_v1)
        .unwrap_or_else(|| "https://api.link-ai.tech/v1".into());
    Some(VisionProvider {
        name: "LinkAI".into(),
        model_override,
        backend: VisionBackend::LinkAi { api_key, api_base },
    })
}

fn build_bot_provider(
    config: Arc<ModelsConfig>,
    bot_type: BotType,
    display_name: &str,
    model_override: Option<String>,
) -> Option<VisionProvider> {
    let bot = create_bot(bot_type, config).ok()?;
    Some(VisionProvider {
        name: display_name.to_string(),
        model_override,
        backend: VisionBackend::Bot { bot },
    })
}

fn route_by_provider_id(
    config: Arc<ModelsConfig>,
    provider_id: &str,
    user_model: &str,
) -> Option<Vec<VisionProvider>> {
    let model = Some(user_model.to_string());
    let p = match provider_id {
        "openai" => build_openai(&config, model),
        "linkai" => build_linkai(&config, model),
        "moonshot" => build_bot_provider(config, BotType::Moonshot, "Moonshot", model),
        "doubao" => build_bot_provider(config, BotType::Doubao, "Doubao", model),
        "dashscope" => build_bot_provider(config, BotType::QwenDashscope, "DashScope", model),
        "claudeAPI" => build_bot_provider(config, BotType::ClaudeApi, "Claude", model),
        "gemini" => build_bot_provider(config, BotType::Gemini, "Gemini", model),
        "qianfan" => build_bot_provider(config, BotType::Qianfan, "Qianfan", model),
        "zhipu" => build_bot_provider(config, BotType::ZhipuAi, "ZhipuAI", model),
        "minimax" => build_bot_provider(config, BotType::Minimax, "MiniMax", model),
        _ => None,
    }?;
    Some(vec![p])
}

fn route_by_model_name(config: Arc<ModelsConfig>, user_model: &str) -> Option<Vec<VisionProvider>> {
    let lower = user_model.to_lowercase();
    if lower.starts_with("gpt-")
        || lower.starts_with("o1-")
        || lower.starts_with("o3-")
        || lower.starts_with("o4-")
        || lower.starts_with("chatgpt-")
    {
        let mut out = Vec::new();
        if config.use_linkai.unwrap_or(false) {
            if let Some(p) = build_linkai(&config, Some(user_model.to_string())) {
                out.push(p);
            }
            if let Some(p) = build_openai(&config, Some(user_model.to_string())) {
                out.push(p);
            }
        } else {
            if let Some(p) = build_openai(&config, Some(user_model.to_string())) {
                out.push(p);
            }
            if let Some(p) = build_linkai(&config, Some(user_model.to_string())) {
                out.push(p);
            }
        }
        return if out.is_empty() { None } else { Some(out) };
    }

    let display = infer_provider_from_model(user_model)?;
    let bot_type = match display {
        "Moonshot" => BotType::Moonshot,
        "Doubao" => BotType::Doubao,
        "DashScope" => BotType::QwenDashscope,
        "Claude" => BotType::ClaudeApi,
        "Gemini" => BotType::Gemini,
        "Qianfan" => BotType::Qianfan,
        "ZhipuAI" => BotType::ZhipuAi,
        "MiniMax" => BotType::Minimax,
        _ => return None,
    };
    build_bot_provider(config, bot_type, display, Some(user_model.to_string())).map(|p| vec![p])
}

fn append_discoverable(config: Arc<ModelsConfig>, providers: &mut Vec<VisionProvider>) {
    let existing: std::collections::HashSet<String> =
        providers.iter().map(|p| p.name.clone()).collect();
    let entries: &[(&str, BotType, &str, &str)] = &[
        ("moonshot_api_key", BotType::Moonshot, "moonshot-v1-8k", "Moonshot"),
        ("ark_api_key", BotType::Doubao, "doubao-seed-2-0-pro", "Doubao"),
        ("dashscope_api_key", BotType::QwenDashscope, "qwen-plus", "DashScope"),
        ("claude_api_key", BotType::ClaudeApi, "claude-sonnet-4-20250514", "Claude"),
        ("gemini_api_key", BotType::Gemini, "gemini-2.0-flash", "Gemini"),
        ("qianfan_api_key", BotType::Qianfan, "ernie-4.5-turbo-vl", "Qianfan"),
        ("zhipu_ai_api_key", BotType::ZhipuAi, "glm-4-flash", "ZhipuAI"),
        ("minimax_api_key", BotType::Minimax, "abab6.5-chat", "MiniMax"),
    ];
    for (key_field, bot_type, default_model, name) in entries {
        if existing.contains(*name) {
            continue;
        }
        let key = match *key_field {
            "moonshot_api_key" => config.moonshot_api_key.as_deref(),
            "ark_api_key" => config.ark_api_key.as_deref(),
            "dashscope_api_key" => config.dashscope_api_key.as_deref(),
            "claude_api_key" => config.claude_api_key.as_deref(),
            "gemini_api_key" => config.gemini_api_key.as_deref(),
            "qianfan_api_key" => config.qianfan_api_key.as_deref(),
            "zhipu_ai_api_key" => config.zhipu_ai_api_key.as_deref(),
            "minimax_api_key" => config.minimax_api_key.as_deref(),
            _ => None,
        };
        if !key.is_some_and(|k| valid_key(k)) {
            continue;
        }
        if let Some(p) = build_bot_provider(
            config.clone(),
            *bot_type,
            name,
            Some(default_model.to_string()),
        ) {
            providers.push(p);
        }
    }
}

/// Build ordered provider list (mirrors `Vision._resolve_providers`).
pub fn resolve_providers(config: Arc<ModelsConfig>) -> Vec<VisionProvider> {
    let user_model = user_vision_model(&config);
    let user_provider = user_vision_provider(&config);
    let mut providers = Vec::new();

    if let (Some(pid), Some(ref um)) = (user_provider.as_deref(), user_model.as_deref()) {
        if let Some(mut preferred) = route_by_provider_id(config.clone(), pid, um) {
            providers.append(&mut preferred);
        }
    }
    if providers.is_empty() {
        if let Some(ref um) = user_model {
            if let Some(mut preferred) = route_by_model_name(config.clone(), um) {
                providers.append(&mut preferred);
            }
        }
    }

    let existing: std::collections::HashSet<String> =
        providers.iter().map(|p| p.name.clone()).collect();
    let mut fallback = Vec::new();
    let use_linkai = config.use_linkai.unwrap_or(false);
    if use_linkai {
        if let Some(p) = build_linkai(&config, user_model.clone()) {
            fallback.push(p);
        }
        if let Some(p) = build_openai(&config, None) {
            fallback.push(p);
        }
    } else {
        if let Some(p) = build_openai(&config, None) {
            fallback.push(p);
        }
        if let Some(p) = build_linkai(&config, user_model.clone()) {
            fallback.push(p);
        }
    }
    append_discoverable(config.clone(), &mut fallback);
    for p in fallback {
        if !existing.contains(&p.name) {
            providers.push(p);
        }
    }
    providers
}

pub fn default_vision_model() -> &'static str {
    DEFAULT_MODEL
}
