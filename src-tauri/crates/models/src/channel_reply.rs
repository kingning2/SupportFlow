//! Shared channel `reply()` helpers (clear memory / reload config).

use crate::bridge::Reply;
use crate::config::ModelsConfig;
use crate::session::SessionManager;

/// Handle `#清除记忆` / `#清除所有` / `#更新配置` — mirrors bot `reply()` prefixes.
pub fn try_admin_commands(
    query: &str,
    config: &ModelsConfig,
    sessions: &SessionManager,
    session_id: &str,
) -> Option<Reply> {
    let clear_cmds = config.clear_memory_commands();
    if clear_cmds.iter().any(|c| c == query) {
        sessions.clear_session(session_id);
        return Some(Reply::info("记忆已清除"));
    }
    if query == "#清除所有" {
        sessions.clear_all_sessions();
        return Some(Reply::info("所有人记忆已清除"));
    }
    if query == "#更新配置" {
        return Some(Reply::info("配置已更新"));
    }
    None
}

/// Build final reply from `reply_text` result dict shape.
pub fn reply_from_text_result(result: &ReplyTextResult) -> Reply {
    if result.completion_tokens == 0 && !result.content.is_empty() {
        Reply::error(&result.content)
    } else if result.completion_tokens > 0 {
        Reply::text(&result.content)
    } else {
        Reply::error(&result.content)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReplyTextResult {
    pub total_tokens: u32,
    pub completion_tokens: u32,
    pub content: String,
}
