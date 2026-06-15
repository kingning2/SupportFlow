//! Model name → `BotType` routing (`bridge/bridge.py` + `agent_bridge.AgentLLMModel`).

use crate::config::config::ModelsConfig;
use crate::config::const_::BotType;

/// Resolve chat bot type from config and model name (mirrors Python `Bridge.__init__` / `AgentLLMModel._resolve_bot_type`).
pub fn resolve_bot_type(config: &ModelsConfig) -> Result<BotType, String> {
    if config.use_linkai.unwrap_or(false) && config.has_linkai_key() {
        return Ok(BotType::Linkai);
    }

    if let Ok(explicit) = config.bot_type() {
        if !config.bot_type.is_empty() {
            return Ok(explicit);
        }
    }

    let model = config.model_or("");
    if model.is_empty() {
        return Ok(BotType::Openai);
    }

    let lowered = model.to_lowercase();

    if model == "text-davinci-003" {
        return Ok(BotType::OpenAi);
    }
    if matches!(model.as_str(), "wenxin" | "wenxin-4") {
        return Ok(BotType::Baidu);
    }
    if model == "xunfei" {
        return Ok(BotType::Xunfei);
    }
    if matches!(
        model.as_str(),
        "qwen" | "qwen-turbo" | "qwen-plus" | "qwen-max"
    ) {
        return Ok(BotType::QwenDashscope);
    }
    if lowered.starts_with("qwen") || lowered.starts_with("qwq") || lowered.starts_with("qvq") {
        return Ok(BotType::QwenDashscope);
    }
    if lowered.starts_with("gemini") {
        return Ok(BotType::Gemini);
    }
    if lowered.starts_with("glm") {
        return Ok(BotType::ZhipuAi);
    }
    if lowered.starts_with("claude") {
        return Ok(BotType::ClaudeApi);
    }
    if matches!(
        model.as_str(),
        "moonshot" | "moonshot-v1-8k" | "moonshot-v1-32k" | "moonshot-v1-128k"
    ) || lowered.starts_with("kimi")
    {
        return Ok(BotType::Moonshot);
    }
    if lowered.starts_with("doubao") {
        return Ok(BotType::Doubao);
    }
    if lowered.starts_with("deepseek") {
        return Ok(BotType::Deepseek);
    }
    if lowered == "qianfan" || lowered.starts_with("ernie") {
        return Ok(BotType::Qianfan);
    }
    if model == "modelscope" {
        return Ok(BotType::Modelscope);
    }
    if lowered.starts_with("minimax") || matches!(model.as_str(), "abab6.5-chat" | "abab6.5") {
        return Ok(BotType::Minimax);
    }

    config.bot_type()
}

/// Pick ASR provider when `voice_to_text` is unset (Python `Bridge._auto_pick_voice_to_text`).
pub fn auto_pick_voice_to_text(config: &ModelsConfig) -> &'static str {
    if config.has_openai_key() {
        return "openai";
    }
    if config.has_dashscope_key() {
        return "dashscope";
    }
    if config.has_zhipu_key() {
        return "zhipu";
    }
    if config.has_linkai_key() {
        return "linkai";
    }
    "openai"
}
