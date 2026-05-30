//! `models/modelscope/modelscope_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct ModelScopeBot;
    default_model: "qwen-max",
    session: SessionClass::StandardChatLen,
    api_key: |c| c.open_ai_api_key.clone().unwrap_or_default(),
    api_base: |c| "https://api-inference.modelscope.cn/v1".to_string(),
}
