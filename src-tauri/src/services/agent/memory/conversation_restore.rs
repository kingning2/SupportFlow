//! Restore LLM context from persisted messages (`AgentInitializer._filter_text_only_messages`).

use serde_json::{json, Value};

fn extract_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.trim().to_string(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| {
                b.get("type")
                    .and_then(|t| t.as_str())
                    .filter(|t| *t == "text")
                    .and_then(|_| b.get("text"))
                    .and_then(|t| t.as_str())
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn is_real_user_msg(msg: &Value) -> bool {
    if msg.get("role").and_then(|r| r.as_str()) != Some("user") {
        return false;
    }
    let content = msg.get("content").unwrap_or(&Value::Null);
    if let Value::Array(blocks) = content {
        if blocks.iter().any(|b| {
            b.get("type")
                .and_then(|t| t.as_str())
                .is_some_and(|t| t == "tool_result")
        }) {
            return false;
        }
    }
    !extract_text(content).is_empty()
}

/// Keep one user + one assistant text message per visible turn.
pub fn filter_text_only_messages(messages: &[Value]) -> Vec<Value> {
    struct Turn {
        user: Value,
        assistants: Vec<String>,
    }

    let mut turns: Vec<Turn> = Vec::new();
    let mut current: Option<Turn> = None;

    for msg in messages {
        if is_real_user_msg(msg) {
            if let Some(t) = current.take() {
                turns.push(t);
            }
            current = Some(Turn {
                user: msg.clone(),
                assistants: Vec::new(),
            });
        } else if let Some(ref mut t) = current {
            if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                let text = extract_text(msg.get("content").unwrap_or(&Value::Null));
                if !text.is_empty() {
                    t.assistants.push(text);
                }
            }
        }
    }
    if let Some(t) = current {
        turns.push(t);
    }

    let mut out = Vec::new();
    for turn in turns {
        let user_text = extract_text(turn.user.get("content").unwrap_or(&Value::Null));
        if user_text.is_empty() {
            continue;
        }
        out.push(json!({
            "role": "user",
            "content": [{"type": "text", "text": user_text}]
        }));
        if let Some(reply) = turn.assistants.last() {
            out.push(json!({
                "role": "assistant",
                "content": [{"type": "text", "text": reply}]
            }));
        }
    }
    out
}

pub fn strip_thinking_blocks(messages: &[Value]) -> Vec<Value> {
    messages
        .iter()
        .map(|msg| {
            if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") {
                return msg.clone();
            }
            let Some(Value::Array(blocks)) = msg.get("content") else {
                return msg.clone();
            };
            let filtered: Vec<Value> = blocks
                .iter()
                .filter(|b| b.get("type").and_then(|t| t.as_str()) != Some("thinking"))
                .cloned()
                .collect();
            if filtered.len() == blocks.len() {
                return msg.clone();
            }
            let mut copy = msg.clone();
            if let Some(obj) = copy.as_object_mut() {
                obj.insert("content".into(), Value::Array(filtered));
            }
            copy
        })
        .collect()
}
