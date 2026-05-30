//! `models/openai/open_ai_bot.py`

use std::sync::Arc;

use async_trait::async_trait;

use crate::config::ModelsConfig;
use crate::openai::OpenAiHttpClient;
use crate::openai_compatible::{ApiConfig, OpenAICompatibleBot};
use crate::session::SessionManager;

#[derive(Debug)]
pub struct OpenAiBot {
    config: Arc<ModelsConfig>,
    pub sessions: SessionManager,
    client: OpenAiHttpClient,
}

impl OpenAiBot {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
        let model = config.model_or("gpt-4o-mini");
        let sessions =
            crate::provider::session_manager(&config, crate::session::SessionClass::OpenAi, &model);
        let client = OpenAiHttpClient::new(
            config.open_ai_api_key.clone(),
            config.open_ai_api_base.clone(),
        )
        .with_timeout_secs(config.request_timeout_secs());
        Self {
            config,
            sessions,
            client,
        }
    }
}

#[async_trait]
impl OpenAICompatibleBot for OpenAiBot {
    fn get_api_config(&self) -> ApiConfig {
        ApiConfig {
            api_key: self.config.open_ai_api_key.clone().unwrap_or_default(),
            api_base: self
                .config
                .open_ai_api_base
                .clone()
                .unwrap_or_else(|| crate::openai::DEFAULT_API_BASE.to_string()),
            model: self.config.model_or("gpt-4o-mini"),
            default_temperature: self.config.temperature_or(0.9),
            default_top_p: self.config.top_p_or(1.0),
            default_frequency_penalty: self.config.frequency_penalty_or(0.0),
            default_presence_penalty: self.config.presence_penalty_or(0.0),
        }
    }

    fn http_client(&self) -> &OpenAiHttpClient {
        &self.client
    }
}
