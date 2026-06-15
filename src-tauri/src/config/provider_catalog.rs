//! Provider metadata and `config.json` patch helpers for the models console.

use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::Path;

use crate::config::config::ModelsConfig;
use crate::config::const_::BotType;

#[derive(Debug, Clone, Copy)]
pub struct ProviderMeta {
    pub id: &'static str,
    pub aliases: &'static [&'static str],
    pub api_key_field: &'static str,
    pub api_base_field: Option<&'static str>,
    pub api_base_default: Option<&'static str>,
    pub bot_type_value: &'static str,
    pub models: &'static [&'static str],
}

const DEEPSEEK_MODELS: &[&str] = &[
    BotType::DEEPSEEK_V4_FLASH,
    BotType::DEEPSEEK_V4_PRO,
    BotType::DEEPSEEK_CHAT,
    BotType::DEEPSEEK_REASONER,
];

const OPENAI_MODELS: &[&str] = &[
    "gpt-5.5",
    "gpt-5.4",
    "gpt-5.4-mini",
    "gpt-4o",
    "gpt-4o-mini",
];

const CLAUDE_MODELS: &[&str] = &["claude-sonnet-4-6", "claude-opus-4-8", "claude-sonnet-4-5"];

const GEMINI_MODELS: &[&str] = &["gemini-2.5-flash", "gemini-2.5-pro", "gemini-2.0-flash"];

const ZHIPU_MODELS: &[&str] = &["glm-5", "glm-4.7", "glm-4-flash"];

const DASHSCOPE_MODELS: &[&str] = &["qwen-max", "qwen-plus", "qwen-turbo"];

const DOUBAO_MODELS: &[&str] = &["doubao-seed-2-pro", "doubao-seed-2-code"];

const MOONSHOT_MODELS: &[&str] = &["kimi-k2", "moonshot-v1-8k"];

const MINIMAX_MODELS: &[&str] = &["MiniMax-M2.5", "abab6.5s-chat"];

pub const PROVIDER_METAS: &[ProviderMeta] = &[
    ProviderMeta {
        id: "deepseek",
        aliases: &[],
        api_key_field: "deepseek_api_key",
        api_base_field: Some("deepseek_api_base"),
        api_base_default: Some("https://api.deepseek.com/v1"),
        bot_type_value: "deepseek",
        models: DEEPSEEK_MODELS,
    },
    ProviderMeta {
        id: "openAI",
        aliases: &["openai", "chatGPT", "chatgpt"],
        api_key_field: "open_ai_api_key",
        api_base_field: Some("open_ai_api_base"),
        api_base_default: Some("https://api.openai.com/v1"),
        bot_type_value: "chatGPT",
        models: OPENAI_MODELS,
    },
    ProviderMeta {
        id: "chatGPTOnAzure",
        aliases: &["azure"],
        api_key_field: "open_ai_api_key",
        api_base_field: Some("open_ai_api_base"),
        api_base_default: Some("https://api.openai.com/v1"),
        bot_type_value: "chatGPTOnAzure",
        models: OPENAI_MODELS,
    },
    ProviderMeta {
        id: "claudeAPI",
        aliases: &["claudeapi", "claude"],
        api_key_field: "claude_api_key",
        api_base_field: Some("claude_api_base"),
        api_base_default: Some("https://api.anthropic.com/v1"),
        bot_type_value: "claudeAPI",
        models: CLAUDE_MODELS,
    },
    ProviderMeta {
        id: "gemini",
        aliases: &[],
        api_key_field: "gemini_api_key",
        api_base_field: Some("gemini_api_base"),
        api_base_default: Some("https://generativelanguage.googleapis.com"),
        bot_type_value: "gemini",
        models: GEMINI_MODELS,
    },
    ProviderMeta {
        id: "zhipu",
        aliases: &["zhipuai", "glm-4"],
        api_key_field: "zhipu_ai_api_key",
        api_base_field: None,
        api_base_default: None,
        bot_type_value: "zhipu",
        models: ZHIPU_MODELS,
    },
    ProviderMeta {
        id: "moonshot",
        aliases: &[],
        api_key_field: "moonshot_api_key",
        api_base_field: None,
        api_base_default: None,
        bot_type_value: "moonshot",
        models: MOONSHOT_MODELS,
    },
    ProviderMeta {
        id: "doubao",
        aliases: &[],
        api_key_field: "ark_api_key",
        api_base_field: Some("ark_base_url"),
        api_base_default: Some("https://ark.cn-beijing.volces.com/api/v3"),
        bot_type_value: "doubao",
        models: DOUBAO_MODELS,
    },
    ProviderMeta {
        id: "dashscope",
        aliases: &["qwen"],
        api_key_field: "dashscope_api_key",
        api_base_field: None,
        api_base_default: None,
        bot_type_value: "dashscope",
        models: DASHSCOPE_MODELS,
    },
    ProviderMeta {
        id: "minimax",
        aliases: &[],
        api_key_field: "minimax_api_key",
        api_base_field: None,
        api_base_default: None,
        bot_type_value: "minimax",
        models: MINIMAX_MODELS,
    },
    ProviderMeta {
        id: "linkai",
        aliases: &[],
        api_key_field: "linkai_api_key",
        api_base_field: None,
        api_base_default: None,
        bot_type_value: "linkai",
        models: &[],
    },
    ProviderMeta {
        id: "custom",
        aliases: &[],
        api_key_field: "custom_api_key",
        api_base_field: Some("custom_api_base"),
        api_base_default: Some(""),
        bot_type_value: "custom",
        models: &[],
    },
];

pub fn find_provider_meta(provider_id: &str) -> Option<&'static ProviderMeta> {
    let trimmed = provider_id.trim();
    PROVIDER_METAS.iter().find(|m| {
        m.id.eq_ignore_ascii_case(trimmed)
            || m.aliases.iter().any(|a| a.eq_ignore_ascii_case(trimmed))
    })
}

pub fn mask_api_key(value: &str) -> String {
    let v = value.trim();
    if v.is_empty() {
        return String::new();
    }
    if v.len() <= 8 {
        return v.to_string();
    }
    format!("{}****{}", &v[..4], &v[v.len() - 4..])
}

fn config_string(config: &ModelsConfig, field: &str) -> Option<String> {
    match field {
        "open_ai_api_key" => config.open_ai_api_key.clone(),
        "open_ai_api_base" => config.open_ai_api_base.clone(),
        "deepseek_api_key" => config.deepseek_api_key.clone(),
        "deepseek_api_base" => config.deepseek_api_base.clone(),
        "claude_api_key" => config.claude_api_key.clone(),
        "claude_api_base" => config.claude_api_base.clone(),
        "gemini_api_key" => config.gemini_api_key.clone(),
        "gemini_api_base" => config.gemini_api_base.clone(),
        "zhipu_ai_api_key" => config.zhipu_ai_api_key.clone(),
        "moonshot_api_key" => config.moonshot_api_key.clone(),
        "ark_api_key" => config.ark_api_key.clone(),
        "ark_base_url" => config.ark_base_url.clone(),
        "dashscope_api_key" => config.dashscope_api_key.clone(),
        "minimax_api_key" => config.minimax_api_key.clone(),
        "linkai_api_key" => config.linkai_api_key.clone(),
        "custom_api_key" => config.custom_api_key.clone(),
        "custom_api_base" => config.custom_api_base.clone(),
        _ => None,
    }
}

pub fn read_config_root(path: &Path) -> Result<Value, String> {
    if !path.is_file() {
        return Ok(Value::Object(Map::new()));
    }
    let text = crate::io::read_to_string(path).map_err(|e| format!("read config: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("parse config: {e}"))
}

pub fn write_config_root(path: &Path, root: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        crate::io::create_dir_all(parent).map_err(|e| format!("create config dir: {e}"))?;
    }
    let text = serde_json::to_string_pretty(root).map_err(|e| format!("serialize config: {e}"))?;
    crate::io::write(path, text).map_err(|e| format!("write config: {e}"))
}

pub fn patch_config_file(path: &Path, fields: &HashMap<String, Value>) -> Result<(), String> {
    let mut root = read_config_root(path)?;
    let obj = root
        .as_object_mut()
        .ok_or_else(|| "config root must be a JSON object".to_string())?;
    for (k, v) in fields {
        obj.insert(k.clone(), v.clone());
    }
    write_config_root(path, &root)
}

pub fn clear_provider_credentials(path: &Path, meta: &ProviderMeta) -> Result<(), String> {
    let mut fields = HashMap::new();
    fields.insert(meta.api_key_field.to_string(), json!(""));
    if let Some(base) = meta.api_base_field {
        fields.insert(base.to_string(), json!(""));
    }
    patch_config_file(path, &fields)
}

pub fn update_provider_credentials(
    path: &Path,
    meta: &ProviderMeta,
    api_key: Option<&str>,
    api_base: Option<&str>,
    api_base_set: bool,
) -> Result<bool, String> {
    let mut fields = HashMap::new();
    let mut changed = false;

    if let Some(key) = api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            fields.insert(meta.api_key_field.to_string(), json!(trimmed));
            changed = true;
        }
    }

    if api_base_set {
        if let Some(base_field) = meta.api_base_field {
            let value = api_base.unwrap_or("").trim();
            fields.insert(base_field.to_string(), json!(value));
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }
    patch_config_file(path, &fields)?;
    Ok(true)
}

pub fn set_chat_model(
    path: &Path,
    provider_id: Option<&str>,
    model: Option<&str>,
) -> Result<bool, String> {
    let mut fields = HashMap::new();
    let mut changed = false;

    if let Some(pid) = provider_id {
        let trimmed = pid.trim();
        if !trimmed.is_empty() {
            let meta = find_provider_meta(trimmed)
                .ok_or_else(|| format!("unknown provider: {trimmed}"))?;
            fields.insert("bot_type".to_string(), json!(meta.bot_type_value));
            changed = true;
            let use_linkai = meta.id.eq_ignore_ascii_case("linkai");
            fields.insert("use_linkai".to_string(), json!(use_linkai));
        }
    }

    if let Some(m) = model {
        let trimmed = m.trim();
        if !trimmed.is_empty() {
            fields.insert("model".to_string(), json!(trimmed));
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }
    patch_config_file(path, &fields)?;
    Ok(true)
}

#[derive(Debug, Clone)]
pub struct ProviderDetail {
    pub id: String,
    pub configured: bool,
    pub is_active: bool,
    pub api_base: Option<String>,
    pub api_base_default: Option<String>,
    pub has_api_base: bool,
    pub api_key_masked: Option<String>,
    pub models: Vec<String>,
    pub bot_type_value: String,
}

pub fn build_provider_details(config: &ModelsConfig) -> Vec<ProviderDetail> {
    use crate::config::catalog::list_providers;

    list_providers(config)
        .into_iter()
        .map(|p| {
            let Some(meta) = find_provider_meta(&p.id) else {
                return ProviderDetail {
                    id: p.id,
                    configured: p.configured,
                    is_active: p.is_active,
                    api_base: None,
                    api_base_default: None,
                    has_api_base: false,
                    api_key_masked: None,
                    models: Vec::new(),
                    bot_type_value: String::new(),
                };
            };

            let key_trimmed = config_string(config, meta.api_key_field)
                .unwrap_or_default()
                .trim()
                .to_string();
            let api_base = meta.api_base_field.and_then(|f| config_string(config, f));
            let api_key_masked = if key_trimmed.is_empty() {
                None
            } else {
                Some(mask_api_key(&key_trimmed))
            };

            ProviderDetail {
                id: p.id,
                configured: p.configured,
                is_active: p.is_active,
                api_base,
                api_base_default: meta.api_base_default.map(str::to_string),
                has_api_base: meta.api_base_field.is_some(),
                api_key_masked,
                models: meta.models.iter().map(|s| (*s).to_string()).collect(),
                bot_type_value: meta.bot_type_value.to_string(),
            }
        })
        .collect()
}
