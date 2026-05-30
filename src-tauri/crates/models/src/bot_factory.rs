//! `models/bot_factory.py` — `create_bot(bot_type)`.

use std::sync::Arc;

use crate::baidu::BaiduWenxinBot;
use crate::chatgpt::{AzureChatGptBot, ChatGptBot};
use crate::claudeapi::ClaudeApiBot;
use crate::config::ModelsConfig;
use crate::const_::BotType;
use crate::dashscope::DashscopeBot;
use crate::deepseek::DeepSeekBot;
use crate::doubao::DoubaoBot;
use crate::gemini::GoogleGeminiBot;
use crate::linkai::LinkAiBot;
use crate::minimax::MinimaxBot;
use crate::modelscope::ModelScopeBot;
use crate::moonshot::MoonshotBot;
use crate::openai::OpenAiBot;
use crate::openai_compatible::OpenAICompatibleBot;
use crate::qianfan::QianfanBot;
use crate::xunfei::XunfeiBot;
use crate::zhipuai::ZhipuAiBot;

/// Type-erased bot instance for agent / channel use.
pub type BotHandle = Arc<dyn OpenAICompatibleBot>;

/// Create a bot for the given `bot_type` (mirrors Python `create_bot`).
pub fn create_bot(bot_type: BotType, config: Arc<ModelsConfig>) -> Result<BotHandle, String> {
    let bot: Arc<dyn OpenAICompatibleBot> = match bot_type {
        BotType::Baidu => Arc::new(BaiduWenxinBot::new(config)),
        BotType::Deepseek => Arc::new(DeepSeekBot::new(config)),
        BotType::Qianfan => Arc::new(QianfanBot::new(config)),
        BotType::Openai | BotType::ChatGpt | BotType::Custom => Arc::new(ChatGptBot::new(config)),
        BotType::OpenAi => Arc::new(OpenAiBot::new(config)),
        BotType::ChatGptOnAzure => Arc::new(AzureChatGptBot::new(config)),
        BotType::Xunfei => Arc::new(XunfeiBot::new(config)),
        BotType::Linkai => Arc::new(LinkAiBot::new(config)),
        BotType::ClaudeApi => Arc::new(ClaudeApiBot::new(config)),
        BotType::Qwen | BotType::QwenDashscope => Arc::new(DashscopeBot::new(config)),
        BotType::Gemini => Arc::new(GoogleGeminiBot::new(config)),
        BotType::ZhipuAi => Arc::new(ZhipuAiBot::new(config)),
        BotType::Moonshot => Arc::new(MoonshotBot::new(config)),
        BotType::Minimax => Arc::new(MinimaxBot::new(config)),
        BotType::Modelscope => Arc::new(ModelScopeBot::new(config)),
        BotType::Doubao => Arc::new(DoubaoBot::new(config)),
    };
    Ok(bot)
}

/// Parse `config.bot_type` and create the corresponding bot.
pub fn create_bot_from_config(config: Arc<ModelsConfig>) -> Result<BotHandle, String> {
    let bot_type = config.bot_type()?;
    create_bot(bot_type, config)
}
