//! Generic OpenAI-compatible vendor bot (reduces duplication across providers).

use std::sync::Arc;

use async_trait::async_trait;

use crate::config::ModelsConfig;
use crate::openai::OpenAiHttpClient;
use crate::openai_compatible::{ApiConfig, OpenAICompatibleBot};
use crate::provider::{api_config, http_client, session_manager};
use crate::session::{SessionClass, SessionManager};

#[derive(Debug)]
pub struct OpenAiVendorBot {
    pub sessions: SessionManager,
    client: OpenAiHttpClient,
    api: ApiConfig,
}

impl OpenAiVendorBot {
    pub fn build(
        config: Arc<ModelsConfig>,
        api_key: String,
        api_base: String,
        default_model: &str,
        session_class: SessionClass,
    ) -> Self {
        let model = config.model_or(default_model);
        Self {
            sessions: session_manager(&config, session_class, default_model),
            client: http_client(&config, Some(api_key.clone()), Some(api_base.clone())),
            api: api_config(api_key, api_base, model, &config),
        }
    }
}

#[async_trait]
impl OpenAICompatibleBot for OpenAiVendorBot {
    fn get_api_config(&self) -> ApiConfig {
        self.api.clone()
    }

    fn http_client(&self) -> &OpenAiHttpClient {
        &self.client
    }
}

macro_rules! vendor_bot {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident;
        default_model: $default_model:expr,
        session: $session:expr,
        api_key: |$cfg_k:ident| $key_expr:expr,
        api_base: |$cfg_b:ident| $base_expr:expr,
    ) => {
        $(#[$meta])*
        $vis struct $name(crate::vendor_bot::OpenAiVendorBot);

        impl $name {
            pub fn new(config: std::sync::Arc<crate::config::ModelsConfig>) -> Self {
                let $cfg_k = &config;
                let api_key: String = $key_expr;
                let api_base: String = ($base_expr);
                Self(crate::vendor_bot::OpenAiVendorBot::build(
                    config,
                    api_key,
                    api_base,
                    $default_model,
                    $session,
                ))
            }
        }

        #[async_trait::async_trait]
        impl crate::openai_compatible::OpenAICompatibleBot for $name {
            fn get_api_config(&self) -> crate::openai_compatible::ApiConfig {
                self.0.get_api_config()
            }

            fn http_client(&self) -> &crate::openai::OpenAiHttpClient {
                self.0.http_client()
            }
        }
    };
}

pub(crate) use vendor_bot;
