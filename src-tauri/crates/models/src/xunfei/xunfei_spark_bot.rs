//! `models/xunfei/xunfei_spark_bot.py` — WebSocket API; OpenAI-compat stub for factory wiring.

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct XunfeiBot;
    default_model: "generalv3.5",
    session: SessionClass::StandardChatLen,
    api_key: |c| c.open_ai_api_key.clone().unwrap_or_default(),
    api_base: |c| "https://spark-api-open.xf-yun.com/v1".to_string(),
}
