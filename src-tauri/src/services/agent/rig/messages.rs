//! 在 rig `Message` 与既有 JSON 消息格式之间转换。

use rig_core::completion::{AssistantContent, Message};
use rig_core::message::{Text, ToolCall, ToolFunction, UserContent};
use rig_core::OneOrMany;
use serde_json::{json, Value};

/// 将持久化的 JSON 消息列表转为 rig 聊天历史。
pub fn json_messages_to_rig(messages: &[Value]) -> Vec<Message> {
    messages.iter().filter_map(json_message_to_rig).collect()
}

/// 将 rig 聊天历史写回 JSON 消息列表（供会话持久化）。
pub fn rig_messages_to_json(messages: &[Message]) -> Vec<Value> {
    messages.iter().map(rig_message_to_json).collect()
}

fn json_message_to_rig(msg: &Value) -> Option<Message> {
    let role = msg.get("role")?.as_str()?;
    match role {
        "user" => {
            let text = extract_text(msg.get("content").unwrap_or(&Value::Null));
            if text.is_empty() {
                return None;
            }
            Some(Message::user(text))
        }
        "assistant" => {
            let content = msg.get("content")?;
            let mut parts: Vec<AssistantContent> = Vec::new();

            if let Value::Array(blocks) = content {
                for block in blocks {
                    let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match block_type {
                        "text" => {
                            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                if !text.is_empty() {
                                    parts.push(AssistantContent::text(text));
                                }
                            }
                        }
                        "tool_use" => {
                            let id = block
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("tool_call")
                                .to_string();
                            let name = block
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let input = block.get("input").cloned().unwrap_or(json!({}));
                            parts.push(AssistantContent::ToolCall(ToolCall::new(
                                id,
                                ToolFunction {
                                    name,
                                    arguments: input,
                                },
                            )));
                        }
                        _ => {}
                    }
                }
            } else {
                let text = extract_text(content);
                if !text.is_empty() {
                    parts.push(AssistantContent::text(text));
                }
            }

            if parts.is_empty() {
                return None;
            }
            Some(Message::Assistant {
                id: None,
                content: OneOrMany::many(parts)
                    .unwrap_or_else(|_| OneOrMany::one(AssistantContent::text(""))),
            })
        }
        _ => None,
    }
}

fn rig_message_to_json(msg: &Message) -> Value {
    match msg {
        Message::System { content } => json!({
            "role": "system",
            "content": content,
        }),
        Message::User { content } => {
            let text = content
                .iter()
                .filter_map(|c| match c {
                    UserContent::Text(Text { text, .. }) => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            json!({
                "role": "user",
                "content": [{"type": "text", "text": text}],
            })
        }
        Message::Assistant { content, .. } => {
            let blocks: Vec<Value> = content
                .iter()
                .map(|c| match c {
                    AssistantContent::Text(Text { text, .. }) => json!({
                        "type": "text",
                        "text": text,
                    }),
                    AssistantContent::ToolCall(tc) => json!({
                        "type": "tool_use",
                        "id": tc.id,
                        "name": tc.function.name,
                        "input": tc.function.arguments,
                    }),
                    _ => json!({
                        "type": "text",
                        "text": "",
                    }),
                })
                .collect();
            json!({
                "role": "assistant",
                "content": blocks,
            })
        }
    }
}

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
