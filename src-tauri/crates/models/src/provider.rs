//! Shared helpers for vendor bot modules.

use std::sync::Arc;

use crate::config::ModelsConfig;
use crate::openai::OpenAiHttpClient;
use crate::openai_compatible::ApiConfig;
use crate::session::{SessionClass, SessionManager};

pub fn session_manager(
    config: &ModelsConfig,
    session_class: SessionClass,
    default_model: &str,
) -> SessionManager {
    SessionManager::new(config, session_class, default_model)
}

pub fn http_client(
    config: &ModelsConfig,
    api_key: Option<String>,
    api_base: Option<String>,
) -> OpenAiHttpClient {
    OpenAiHttpClient::new(api_key, api_base).with_timeout_secs(config.request_timeout_secs())
}

pub fn api_config(
    api_key: String,
    api_base: String,
    model: String,
    config: &ModelsConfig,
) -> ApiConfig {
    ApiConfig {
        api_key,
        api_base,
        model,
        default_temperature: config.temperature_or(0.9),
        default_top_p: config.top_p_or(1.0),
        default_frequency_penalty: config.frequency_penalty_or(0.0),
        default_presence_penalty: config.presence_penalty_or(0.0),
    }
}

pub type SharedConfig = Arc<ModelsConfig>;
