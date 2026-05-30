//! `models/qianfan/qianfan_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct QianfanBot;
    default_model: "ernie-5.1",
    session: SessionClass::StandardChatBlocks,
    api_key: |c| c.open_ai_api_key.clone().unwrap_or_default(),
    api_base: |c| "https://qianfan.baidubce.com/v2".to_string(),
}
