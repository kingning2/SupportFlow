//! `models/linkai/link_ai_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct LinkAiBot;
    default_model: "gpt-4o-mini",
    session: SessionClass::ChatGpt,
    api_key: |c| c.linkai_api_key.clone().unwrap_or_default(),
    api_base: |c| "https://api.link-ai.tech/v1".to_string(),
}
