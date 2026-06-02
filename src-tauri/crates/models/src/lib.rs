//! LLM `models/` package — provider bots, HTTP client, sessions.
//!
//! Directory layout mirrors the Python repo:
//! `bot.py`, `bot_factory.py`, `openai_compatible_bot.py`, `session_manager.py`,
//! and per-vendor subpackages (`openai/`, `deepseek/`, …).

pub mod bot;
pub mod bot_factory;
pub mod bridge;
pub mod catalog;
pub mod channel_reply;
pub mod config;
pub mod http_proxy;
pub mod const_;
pub mod message_utils;
pub mod openai_compatible;
pub mod provider;
pub mod provider_catalog;
pub mod session;
pub mod session_manager;
mod vendor_bot;

pub mod baidu;
pub mod chatgpt;
pub mod claudeapi;
pub mod dashscope;
pub mod deepseek;
pub mod doubao;
pub mod gemini;
pub mod linkai;
pub mod minimax;
pub mod modelscope;
pub mod moonshot;
pub mod openai;
pub mod qianfan;
pub mod xunfei;
pub mod zhipuai;

pub use bot::{Bot, BotError};
pub use bot_factory::{create_bot, BotHandle};
pub use bridge::{Context, ContextType, Reply, ReplyType};
pub use catalog::{list_providers, provider_configured, ModelProviderDescriptor};
pub use channel_reply::{reply_from_text_result, try_admin_commands, ReplyTextResult};
pub use config::{BrowserConfig, ModelsConfig, ToolsConfig, VisionConfig, WebSearchConfig};
pub use http_proxy::{build_reqwest_client, log_http_proxy_settings, HttpProxySettings};
pub use const_::BotType;
pub use deepseek::DeepSeekBot;
pub use openai_compatible::{
    ApiConfig, CallWithToolsRequest, LlmResult, OpenAICompatibleBot, OpenAICompatibleBotExt,
};
pub use provider_catalog::{
    build_provider_details, clear_provider_credentials, find_provider_meta, mask_api_key,
    patch_config_file, set_chat_model, update_provider_credentials, ProviderDetail, ProviderMeta,
    PROVIDER_METAS,
};
pub use session_manager::{Session, SessionManager};
