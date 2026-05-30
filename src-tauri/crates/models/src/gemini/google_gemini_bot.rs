//! `models/gemini/google_gemini_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct GoogleGeminiBot;
    default_model: "gemini-2.0-flash",
    session: SessionClass::ChatGpt,
    api_key: |c| c.gemini_api_key.clone().unwrap_or_default(),
    api_base: |c| c
        .gemini_api_base
        .clone()
        .unwrap_or_else(|| {
            "https://generativelanguage.googleapis.com/v1beta/openai".to_string()
        }),
}
