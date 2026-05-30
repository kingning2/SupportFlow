//! `models/chatgpt/chat_gpt_bot.py`

use std::sync::Arc;

use async_trait::async_trait;

use crate::config::ModelsConfig;
use crate::openai::OpenAiHttpClient;
use crate::openai_compatible::{ApiConfig, OpenAICompatibleBot};
use crate::provider::{api_config, http_client, session_manager, SharedConfig};
use crate::session::SessionClass;

#[derive(Debug)]
pub struct ChatGptBot {
    config: SharedConfig,
    pub sessions: crate::session_manager::SessionManager,
    client: OpenAiHttpClient,
}

impl ChatGptBot {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
        let (api_key, api_base) = if config.bot_type == "custom" {
            (
                config.custom_api_key.clone().unwrap_or_default(),
                config.custom_api_base.clone(),
            )
        } else {
            (
                config.open_ai_api_key.clone().unwrap_or_default(),
                config.open_ai_api_base.clone(),
            )
        };
        let model = config.model_or("gpt-4o-mini");
        Self {
            sessions: session_manager(&config, SessionClass::ChatGpt, &model),
            client: http_client(&config, Some(api_key.clone()), api_base.clone()),
            config,
        }
    }

    fn resolve_api(&self) -> (String, String) {
        if self.config.bot_type == "custom" {
            (
                self.config.custom_api_key.clone().unwrap_or_default(),
                self.config
                    .custom_api_base
                    .clone()
                    .unwrap_or_else(|| crate::openai::DEFAULT_API_BASE.to_string()),
            )
        } else {
            (
                self.config.open_ai_api_key.clone().unwrap_or_default(),
                self.config
                    .open_ai_api_base
                    .clone()
                    .unwrap_or_else(|| crate::openai::DEFAULT_API_BASE.to_string()),
            )
        }
    }
}

#[async_trait]
impl OpenAICompatibleBot for ChatGptBot {
    fn get_api_config(&self) -> ApiConfig {
        let (api_key, api_base) = self.resolve_api();
        api_config(
            api_key,
            api_base,
            self.config.model_or("gpt-4o-mini"),
            &self.config,
        )
    }

    fn http_client(&self) -> &OpenAiHttpClient {
        &self.client
    }
}

/// Azure OpenAI (same module as Python `AzureChatGPTBot`).
#[derive(Debug)]
pub struct AzureChatGptBot {
    inner: ChatGptBot,
}

impl AzureChatGptBot {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
        Self {
            inner: ChatGptBot::new(config),
        }
    }
}

#[async_trait]
impl OpenAICompatibleBot for AzureChatGptBot {
    fn get_api_config(&self) -> ApiConfig {
        self.inner.get_api_config()
    }

    fn http_client(&self) -> &OpenAiHttpClient {
        self.inner.http_client()
    }
}
