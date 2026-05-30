//! `models/doubao/doubao_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct DoubaoBot;
    default_model: "doubao-pro-32k",
    session: SessionClass::StandardChatLen,
    api_key: |c| c.ark_api_key.clone().unwrap_or_default(),
    api_base: |c| c
        .ark_base_url
        .clone()
        .unwrap_or_else(|| "https://ark.cn-beijing.volces.com/api/v3".to_string()),
}
