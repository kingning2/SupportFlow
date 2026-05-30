//! `models/baidu/baidu_wenxin.py` — placeholder (ERNIE uses non-OpenAI API; stub for factory parity).

use std::sync::Arc;

use async_trait::async_trait;

use crate::config::ModelsConfig;
use crate::openai::OpenAiHttpClient;
use crate::openai_compatible::{ApiConfig, OpenAICompatibleBot};
use crate::provider::{api_config, http_client, session_manager};
use crate::session::SessionClass;

const DEFAULT_BASE: &str = "https://aip.baidubce.com";

#[derive(Debug)]
pub struct BaiduWenxinBot {
    api: ApiConfig,
    client: OpenAiHttpClient,
    pub sessions: crate::session_manager::SessionManager,
}

impl BaiduWenxinBot {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
        let model = config.model_or("ernie-5.1");
        Self {
            api: api_config(
                String::new(),
                DEFAULT_BASE.to_string(),
                model.clone(),
                &config,
            ),
            client: http_client(&config, None, Some(DEFAULT_BASE.to_string())),
            sessions: session_manager(&config, SessionClass::BaiduWenxin, &model),
        }
    }
}

#[async_trait]
impl OpenAICompatibleBot for BaiduWenxinBot {
    fn get_api_config(&self) -> ApiConfig {
        self.api.clone()
    }

    fn http_client(&self) -> &OpenAiHttpClient {
        &self.client
    }
}
