//! `models/claudeapi/claude_api_bot.py`

use crate::vendor_bot::vendor_bot;

use crate::session::SessionClass;

vendor_bot! {
    pub struct ClaudeApiBot;
    default_model: "claude-sonnet-4-6",
    session: SessionClass::ChatGpt,
    api_key: |c| c.claude_api_key.clone().unwrap_or_default(),
    api_base: |c| c
        .claude_api_base
        .clone()
        .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
}
