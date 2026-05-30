//! `models/minimax/minimax_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct MinimaxBot;
    default_model: "abab6.5s-chat",
    session: SessionClass::Minimax,
    api_key: |c| c.minimax_api_key.clone().unwrap_or_default(),
    api_base: |c| "https://api.minimax.chat/v1".to_string(),
}
