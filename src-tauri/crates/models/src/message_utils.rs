//! Subset of `agent/protocol/message_utils.py` used by `openai_compatible_bot.py`.

use serde_json::{json, Value};
use tracing::warn;

/// `drop_orphaned_tool_results_openai`
pub fn drop_orphaned_tool_results_openai(messages: Vec<Value>) -> Vec<Value> {
    let mut known_ids = std::collections::HashSet::new();
    let mut cleaned = Vec::with_capacity(messages.len());

    for msg in messages {
        if msg.get("role") == Some(&Value::String("assistant".into())) {
            if let Some(Value::Array(tool_calls)) = msg.get("tool_calls") {
                for tc in tool_calls {
                    if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                        if !id.is_empty() {
                            known_ids.insert(id.to_string());
                        }
                    }
                }
            }
        }

        if msg.get("role") == Some(&Value::String("tool".into())) {
            let ref_id = msg
                .get("tool_call_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !ref_id.is_empty() && !known_ids.contains(ref_id) {
                warn!(
                    tool_call_id = ref_id,
                    "Dropping orphaned tool result (tool_call_id not in known ids)"
                );
                continue;
            }
        }
        cleaned.push(msg);
    }
    cleaned
}

/// `OpenAICompatibleBot._convert_messages_to_openai_format`
pub fn convert_messages_to_openai_format(messages: Vec<Value>) -> Vec<Value> {
    if messages.is_empty() {
        return vec![];
    }

    let mut has_tool_call_history = false;
    for msg in &messages {
        if msg.get("role") != Some(&Value::String("assistant".into())) {
            continue;
        }
        if msg.get("tool_calls").is_some() {
            has_tool_call_history = true;
            break;
        }
        if let Some(Value::Array(inner)) = msg.get("content") {
            if inner
                .iter()
                .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
            {
                has_tool_call_history = true;
                break;
            }
        }
    }

    let mut openai_messages = Vec::new();

    for msg in messages {
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let content = msg.get("content").cloned();

        if matches!(content, Some(Value::String(_))) {
            if role == "assistant"
                && has_tool_call_history
                && msg.get("reasoning_content").is_none()
            {
                let mut patched = msg.as_object().cloned().unwrap_or_default();
                patched.insert("reasoning_content".into(), json!(""));
                openai_messages.push(Value::Object(patched));
            } else {
                openai_messages.push(msg);
            }
            continue;
        }

        if let Some(Value::Array(blocks)) = content {
            if role == "user"
                && blocks
                    .iter()
                    .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result"))
            {
                let mut text_parts = Vec::new();
                let mut tool_results = Vec::new();
                for block in &blocks {
                    if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                        text_parts.push(block.get("text").and_then(|t| t.as_str()).unwrap_or(""));
                    } else if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                        tool_results.push(block.clone());
                    }
                }
                for block in tool_results {
                    let tool_call_id = block
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if tool_call_id.is_empty() {
                        warn!("tool_result missing tool_use_id, using empty string");
                    }
                    let result_content = block.get("content").cloned().unwrap_or(Value::Null);
                    let content_str = if let Value::String(s) = result_content {
                        s
                    } else {
                        serde_json::to_string(&result_content).unwrap_or_default()
                    };
                    openai_messages.push(json!({
                        "role": "tool",
                        "tool_call_id": tool_call_id,
                        "content": content_str,
                    }));
                }
                if !text_parts.is_empty() {
                    openai_messages.push(json!({
                        "role": "user",
                        "content": text_parts.join(" "),
                    }));
                }
            } else if role == "assistant" {
                let mut text_parts: Vec<String> = Vec::new();
                let mut tool_calls = Vec::new();
                let mut reasoning_parts: Vec<String> = Vec::new();

                for block in blocks {
                    let btype = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match btype {
                        "text" => text_parts.push(
                            block
                                .get("text")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string(),
                        ),
                        "tool_use" => {
                            let tool_id = block.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            if tool_id.is_empty() {
                                warn!(
                                    name = block.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                                    "tool_use missing id"
                                );
                            }
                            let input = block.get("input").cloned().unwrap_or(json!({}));
                            tool_calls.push(json!({
                                "id": tool_id,
                                "type": "function",
                                "function": {
                                    "name": block.get("name").cloned().unwrap_or(Value::Null),
                                    "arguments": serde_json::to_string(&input).unwrap_or_else(|_| "{}".into()),
                                }
                            }));
                        }
                        "thinking" => reasoning_parts.push(
                            block
                                .get("thinking")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string(),
                        ),
                        _ => {}
                    }
                }

                let mut openai_msg = serde_json::Map::new();
                openai_msg.insert("role".into(), json!("assistant"));
                if !text_parts.is_empty() {
                    openai_msg.insert("content".into(), json!(text_parts.join(" ")));
                } else {
                    openai_msg.insert("content".into(), Value::Null);
                }
                if !tool_calls.is_empty() {
                    openai_msg.insert("tool_calls".into(), Value::Array(tool_calls));
                }
                if !reasoning_parts.is_empty() {
                    openai_msg.insert(
                        "reasoning_content".into(),
                        json!(reasoning_parts.join("\n")),
                    );
                } else if has_tool_call_history {
                    openai_msg.insert("reasoning_content".into(), json!(""));
                }
                if let Some(parts) = msg.get("_gemini_raw_parts") {
                    openai_msg.insert("_gemini_raw_parts".into(), parts.clone());
                }
                openai_messages.push(Value::Object(openai_msg));
            } else {
                openai_messages.push(msg);
            }
        } else {
            openai_messages.push(msg);
        }
    }

    drop_orphaned_tool_results_openai(openai_messages)
}
