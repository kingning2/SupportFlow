//! Turn identification and aggressive trim (`agent_stream.py`).

use serde_json::{json, Value};
use tracing::{info, warn};

use crate::protocol::compress_turn_to_text_only;

#[derive(Debug, Clone)]
pub struct Turn {
    pub messages: Vec<Value>,
}

/// Identify complete conversation turns (`_identify_complete_turns`).
pub fn identify_complete_turns(messages: &[Value]) -> Vec<Turn> {
    let mut turns = Vec::new();
    let mut current = Turn {
        messages: Vec::new(),
    };

    for msg in messages {
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let content = msg.get("content").cloned().unwrap_or(Value::Null);

        if role == "user" {
            let mut is_user_query = false;
            if let Some(blocks) = content.as_array() {
                let has_text = blocks
                    .iter()
                    .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"));
                let has_tool_result = blocks
                    .iter()
                    .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result"));
                is_user_query = has_text && !has_tool_result;
            } else if content.is_string() {
                is_user_query = true;
            }

            if is_user_query {
                if !current.messages.is_empty() {
                    turns.push(current);
                }
                current = Turn {
                    messages: vec![msg.clone()],
                };
            } else {
                current.messages.push(msg.clone());
            }
        } else {
            current.messages.push(msg.clone());
        }
    }

    if !current.messages.is_empty() {
        turns.push(current);
    }

    turns
}

/// Aggressive trim on overflow (`_aggressive_trim_for_overflow`). Returns true if worth retrying.
pub fn aggressive_trim_for_overflow(messages: &mut Vec<Value>) -> bool {
    if messages.is_empty() {
        return false;
    }

    let original_count = messages.len();
    const AGGRESSIVE_LIMIT: usize = 10_000;
    const USER_MSG_LIMIT: usize = 10_000;
    let mut truncated = 0u32;

    for msg in messages.iter_mut() {
        let Some(blocks) = msg.get_mut("content").and_then(|c| c.as_array_mut()) else {
            if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                if let Some(s) = msg.get("content").and_then(|c| c.as_str()) {
                    if s.len() > USER_MSG_LIMIT {
                        let new_content = format!(
                            "{}{}",
                            &s[..USER_MSG_LIMIT],
                            format!(
                                "\n\n[Message truncated for context recovery: {} -> {} chars]",
                                s.len(),
                                USER_MSG_LIMIT
                            )
                        );
                        msg.as_object_mut()
                            .expect("msg object")
                            .insert("content".into(), json!(new_content));
                        truncated += 1;
                    }
                }
            }
            continue;
        };

        for block in blocks.iter_mut() {
            if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                if let Some(s) = block.get("content").and_then(|c| c.as_str()) {
                    if s.len() > AGGRESSIVE_LIMIT {
                        let original_len = s.len();
                        let new_s = format!(
                            "{}{}",
                            &s[..AGGRESSIVE_LIMIT],
                            format!(
                                "\n\n[Truncated for context recovery: {} -> {} chars]",
                                original_len, AGGRESSIVE_LIMIT
                            )
                        );
                        block
                            .as_object_mut()
                            .expect("block")
                            .insert("content".into(), json!(new_s));
                        truncated += 1;
                    }
                }
            }
            if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                if let Some(input) = block.get_mut("input").and_then(|i| i.as_object_mut()) {
                    let input_str = serde_json::to_string(input).unwrap_or_default();
                    if input_str.len() > AGGRESSIVE_LIMIT {
                        for val in input.values_mut() {
                            if let Some(s) = val.as_str() {
                                if s.len() > 1000 {
                                    *val = json!(format!(
                                        "{}... [truncated {} chars]",
                                        &s[..1000],
                                        s.len()
                                    ));
                                }
                            }
                        }
                        truncated += 1;
                    }
                }
            }
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(Value::String(text)) = block.get_mut("text") {
                    if text.len() > USER_MSG_LIMIT {
                        let len = text.len();
                        let new_text = format!(
                            "{}{}",
                            &text[..USER_MSG_LIMIT],
                            format!(
                                "\n\n[Message truncated for context recovery: {} -> {} chars]",
                                len, USER_MSG_LIMIT
                            )
                        );
                        *text = new_text;
                        truncated += 1;
                    }
                }
            }
        }
    }

    let turns = identify_complete_turns(messages);
    if turns.len() > 5 {
        let kept: Vec<Value> = turns
            .iter()
            .rev()
            .take(5)
            .rev()
            .flat_map(|t| t.messages.clone())
            .collect();
        let removed = turns.len() - 5;
        *messages = kept;
        info!(
            removed,
            truncated,
            original_count,
            new_len = messages.len(),
            "Aggressive trim: removed old turns"
        );
        return true;
    }

    if truncated > 0 {
        info!(
            truncated,
            turns = turns.len(),
            "Aggressive trim: truncated large blocks"
        );
        return true;
    }

    warn!("Aggressive trim: nothing to trim, will clear history");
    false
}

pub fn turn_to_value(turn: &Turn) -> Value {
    json!({ "messages": turn.messages })
}

pub fn compress_turn(turn: &Turn) -> Turn {
    let v = compress_turn_to_text_only(&turn_to_value(turn));
    Turn {
        messages: v
            .get("messages")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default(),
    }
}
