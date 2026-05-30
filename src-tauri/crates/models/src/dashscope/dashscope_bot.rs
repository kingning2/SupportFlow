//! `models/dashscope/dashscope_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct DashscopeBot;
    default_model: "qwen-max",
    session: SessionClass::Dashscope,
    api_key: |c| c.dashscope_api_key.clone().unwrap_or_default(),
    api_base: |c| "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
}
