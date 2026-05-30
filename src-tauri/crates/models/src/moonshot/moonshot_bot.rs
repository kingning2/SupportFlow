//! `models/moonshot/moonshot_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct MoonshotBot;
    default_model: "moonshot-v1-8k",
    session: SessionClass::StandardChatLen,
    api_key: |c| c.moonshot_api_key.clone().unwrap_or_default(),
    api_base: |c| "https://api.moonshot.cn/v1".to_string(),
}
