//! `discard_exceeding` implementations from provider `*_session.py`.

use serde_json::Value;
use tracing::{debug, warn};

/// Standard chat sessions (DeepSeek, Doubao, Moonshot, …).
pub fn discard_exceeding_standard<F>(
    messages: &mut Vec<Value>,
    max_tokens: u32,
    cur_tokens: Option<u32>,
    mut calc_tokens: F,
) -> u32
where
    F: FnMut(&[Value]) -> u32,
{
    let mut precise = true;
    let mut cur_tokens = if let Some(t) = cur_tokens {
        precise = false;
        t
    } else {
        calc_tokens(messages)
    };

    loop {
        if cur_tokens <= max_tokens {
            return cur_tokens;
        }
        if messages.len() > 2 {
            messages.remove(1);
        } else if messages.len() == 2 && role_at(messages, 1) == Some("assistant") {
            messages.pop();
            cur_tokens = if precise {
                calc_tokens(messages)
            } else {
                cur_tokens.saturating_sub(max_tokens)
            };
            break;
        } else if messages.len() == 2 && role_at(messages, 1) == Some("user") {
            warn!(total_tokens = cur_tokens, "user message exceed max_tokens");
            break;
        } else {
            debug!(
                max_tokens,
                total_tokens = cur_tokens,
                len = messages.len(),
                "discard_exceeding stop"
            );
            break;
        }
        cur_tokens = if precise {
            calc_tokens(messages)
        } else {
            cur_tokens.saturating_sub(max_tokens)
        };
    }
    cur_tokens
}

/// `open_ai_session.discard_exceeding`
pub fn discard_exceeding_openai_legacy<F, S>(
    messages: &mut Vec<Value>,
    max_tokens: u32,
    cur_tokens: Option<u32>,
    mut calc_tokens: F,
    prompt_len_fallback: S,
) -> u32
where
    F: FnMut(&[Value]) -> u32,
    S: Fn(&[Value]) -> u32,
{
    let mut precise = true;
    let mut cur_tokens = if let Some(t) = cur_tokens {
        precise = false;
        t
    } else {
        calc_tokens(messages)
    };

    loop {
        if cur_tokens <= max_tokens {
            return cur_tokens;
        }
        if messages.len() > 1 {
            messages.remove(0);
        } else if messages.len() == 1 && role_at(messages, 0) == Some("assistant") {
            messages.pop();
            cur_tokens = if precise {
                calc_tokens(messages)
            } else {
                prompt_len_fallback(messages)
            };
            break;
        } else if messages.len() == 1 && role_at(messages, 0) == Some("user") {
            warn!(total_tokens = cur_tokens, "user question exceed max_tokens");
            break;
        } else {
            debug!(
                max_tokens,
                total_tokens = cur_tokens,
                len = messages.len(),
                "discard_exceeding stop"
            );
            break;
        }
        cur_tokens = if precise {
            calc_tokens(messages)
        } else {
            prompt_len_fallback(messages)
        };
    }
    cur_tokens
}

/// `baidu_wenxin_session.discard_exceeding`
pub fn discard_exceeding_baidu_wenxin<F>(
    messages: &mut Vec<Value>,
    max_tokens: u32,
    cur_tokens: Option<u32>,
    mut calc_tokens: F,
) -> u32
where
    F: FnMut(&[Value]) -> u32,
{
    let mut precise = true;
    let mut cur_tokens = if let Some(t) = cur_tokens {
        precise = false;
        t
    } else {
        calc_tokens(messages)
    };

    loop {
        if cur_tokens <= max_tokens {
            return cur_tokens;
        }
        if messages.len() >= 2 {
            messages.remove(0);
            messages.remove(0);
        } else {
            debug!(
                max_tokens,
                total_tokens = cur_tokens,
                len = messages.len(),
                "discard_exceeding stop"
            );
            break;
        }
        cur_tokens = if precise {
            calc_tokens(messages)
        } else {
            cur_tokens.saturating_sub(max_tokens)
        };
    }
    cur_tokens
}

/// `minimax_session.discard_exceeding`
pub fn discard_exceeding_minimax<F>(
    messages: &mut Vec<Value>,
    max_tokens: u32,
    cur_tokens: Option<u32>,
    mut calc_tokens: F,
) -> u32
where
    F: FnMut(&[Value]) -> u32,
{
    let mut precise = true;
    let mut cur_tokens = if let Some(t) = cur_tokens {
        precise = false;
        t
    } else {
        calc_tokens(messages)
    };

    loop {
        if cur_tokens <= max_tokens {
            return cur_tokens;
        }
        if messages.len() > 2 {
            messages.remove(1);
        } else if messages.len() == 2 && sender_type_at(messages, 1) == Some("BOT") {
            messages.pop();
            cur_tokens = if precise {
                calc_tokens(messages)
            } else {
                cur_tokens.saturating_sub(max_tokens)
            };
            break;
        } else if messages.len() == 2 && sender_type_at(messages, 1) == Some("USER") {
            warn!(total_tokens = cur_tokens, "user message exceed max_tokens");
            break;
        } else {
            debug!(
                max_tokens,
                total_tokens = cur_tokens,
                len = messages.len(),
                "discard_exceeding stop"
            );
            break;
        }
        cur_tokens = if precise {
            calc_tokens(messages)
        } else {
            cur_tokens.saturating_sub(max_tokens)
        };
    }
    cur_tokens
}

fn role_at(messages: &[Value], idx: usize) -> Option<&str> {
    messages.get(idx)?.get("role")?.as_str()
}

fn sender_type_at(messages: &[Value], idx: usize) -> Option<&str> {
    messages.get(idx)?.get("sender_type")?.as_str()
}
