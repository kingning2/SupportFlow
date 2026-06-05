//! `agent/protocol/message_utils.py` — Claude/OpenAI message sanitizers.

use std::collections::HashSet;

use serde_json::{json, Value};
use tracing::{info, warn};

const SYNTH_TOOL_ERR: &str = "Error: Missing tool_result adjacent to tool_use (session repair). \
    The conversation history was inconsistent; continue from here.";

fn synth_tool_block(tool_use_id: &str) -> Value {
    json!({
        "type": "tool_result",
        "tool_use_id": tool_use_id,
        "content": SYNTH_TOOL_ERR,
        "is_error": true,
    })
}

/// Anthropic adjacency repair — mirrors `_repair_tool_use_adjacency`.
pub fn repair_tool_use_adjacency(messages: &mut Vec<Value>) -> u32 {
    let mut repairs = 0u32;
    let mut i = 0usize;

    while i < messages.len() {
        let Some(role) = role_str(&messages[i]) else {
            i += 1;
            continue;
        };
        if role != "assistant" {
            i += 1;
            continue;
        }

        let Some(content) = content_blocks(&messages[i]) else {
            i += 1;
            continue;
        };

        let required: Vec<String> = content
            .iter()
            .filter_map(|b| {
                if block_type(b) == Some("tool_use") {
                    block_id(b).map(str::to_string)
                } else {
                    None
                }
            })
            .collect();

        if required.is_empty() {
            i += 1;
            continue;
        }

        let req_set: HashSet<&str> = required.iter().map(String::as_str).collect();

        if i + 1 >= messages.len() {
            let blocks: Vec<Value> = required.iter().map(|tid| synth_tool_block(tid)).collect();
            messages.push(json!({
                "role": "user",
                "content": blocks,
            }));
            warn!("Appended synthetic tool_result after trailing assistant tool_use");
            repairs += 1;
            break;
        }

        let next_role = role_str(&messages[i + 1]).unwrap_or("").to_string();
        if next_role != "user" {
            let blocks: Vec<Value> = required.iter().map(|tid| synth_tool_block(tid)).collect();
            messages.insert(
                i + 1,
                json!({
                    "role": "user",
                    "content": blocks,
                }),
            );
            warn!(
                next_role = %next_role,
                "Inserted synthetic tool_result user after tool_use"
            );
            repairs += 1;
            i += 2;
            continue;
        }

        let next_content: Vec<Value> = match content_blocks(&messages[i + 1]) {
            Some(c) => c.to_vec(),
            None => {
                let blocks: Vec<Value> = required.iter().map(|tid| synth_tool_block(tid)).collect();
                messages.insert(
                    i + 1,
                    json!({
                        "role": "user",
                        "content": blocks,
                    }),
                );
                repairs += 1;
                i += 2;
                continue;
            }
        };

        let present: HashSet<&str> = next_content
            .iter()
            .filter_map(|b| {
                if block_type(b) == Some("tool_result") {
                    block_tool_use_id(b)
                } else {
                    None
                }
            })
            .collect();

        if req_set.iter().all(|id| present.contains(id)) {
            i += 1;
            continue;
        }

        let missing: Vec<&str> = required
            .iter()
            .map(String::as_str)
            .filter(|id| !present.contains(id))
            .collect();

        if let Some(obj) = messages[i + 1].as_object_mut() {
            let mut new_content: Vec<Value> =
                missing.iter().map(|tid| synth_tool_block(tid)).collect();
            new_content.extend(next_content);
            obj.insert("content".into(), Value::Array(new_content));
        }

        warn!(
            ?missing,
            "Prepended synthetic tool_result for Anthropic adjacency"
        );
        repairs += missing.len() as u32;
        i += 1;
    }

    repairs
}

/// Validate and fix Claude-format messages in place (`sanitize_claude_messages`).
pub fn sanitize_claude_messages(messages: &mut Vec<Value>) -> u32 {
    if messages.is_empty() {
        return 0;
    }

    let mut removed = 0u32;
    let mut adj_repairs = repair_tool_use_adjacency(messages);

    // 2. Remove leading orphaned tool_result user messages
    loop {
        if messages.is_empty() {
            break;
        }
        let role = role_str(&messages[0]).unwrap_or("");
        if role != "user" {
            break;
        }
        let Some(content) = content_blocks(&messages[0]) else {
            break;
        };
        if has_block_type(content, "tool_result") && !has_block_type(content, "text") {
            warn!("Removing leading orphaned tool_result user message");
            messages.remove(0);
            removed += 1;
        } else {
            break;
        }
    }

    // 3. Iteratively remove unmatched tool_use / tool_result until stable (max 5 passes)
    for _ in 0..5 {
        let (use_ids, result_ids) = collect_tool_ids(messages);
        let bad_use: HashSet<_> = use_ids.difference(&result_ids).cloned().collect();
        let bad_result: HashSet<_> = result_ids.difference(&use_ids).cloned().collect();

        if bad_use.is_empty() && bad_result.is_empty() {
            break;
        }

        let mut pass_removed = 0u32;
        let mut i = 0usize;
        while i < messages.len() {
            let role = role_str(&messages[i]).unwrap_or("").to_string();
            let Some(content) = content_blocks(&messages[i]).map(|s| s.to_vec()) else {
                i += 1;
                continue;
            };

            if role == "assistant"
                && !bad_use.is_empty()
                && content.iter().any(|b| {
                    block_type(b) == Some("tool_use")
                        && block_id(b).is_some_and(|id| bad_use.contains(id))
                })
            {
                warn!("Removing assistant msg with unmatched tool_use");
                messages.remove(i);
                pass_removed += 1;
                continue;
            }

            if role == "user" && !bad_result.is_empty() && has_block_type(&content, "tool_result") {
                let has_bad = content.iter().any(|b| {
                    block_type(b) == Some("tool_result")
                        && block_tool_use_id(b).is_some_and(|id| bad_result.contains(id))
                });
                if has_bad {
                    if !has_block_type(&content, "text") {
                        warn!("Removing user msg with unmatched tool_result");
                        messages.remove(i);
                        pass_removed += 1;
                        continue;
                    }
                    let before = content.len();
                    let filtered: Vec<Value> = content
                        .into_iter()
                        .filter(|b| {
                            !(block_type(b) == Some("tool_result")
                                && block_tool_use_id(b).is_some_and(|id| bad_result.contains(id)))
                        })
                        .collect();
                    let after = filtered.len();
                    if let Some(obj) = messages[i].as_object_mut() {
                        obj.insert("content".into(), Value::Array(filtered));
                    }
                    pass_removed += (before - after) as u32;
                }
            }

            i += 1;
        }

        removed += pass_removed;
        if pass_removed == 0 {
            break;
        }
    }

    // 4. Re-run adjacency repair if something was removed
    if removed > 0 {
        adj_repairs += repair_tool_use_adjacency(messages);
    }

    if removed > 0 {
        info!(removed, "Message validation: removed broken message(s)");
    }
    if adj_repairs > 0 {
        info!(adj_repairs, "Message validation: adjacency repairs");
    }

    removed + adj_repairs
}

/// OpenAI-format sanitizer (`drop_orphaned_tool_results_openai`).
pub fn drop_orphaned_tool_results_openai(messages: Vec<Value>) -> Vec<Value> {
    let mut known_ids: HashSet<String> = HashSet::new();
    let mut cleaned = Vec::with_capacity(messages.len());

    for msg in messages {
        if role_str(&msg) == Some("assistant") {
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

        if role_str(&msg) == Some("tool") {
            let ref_id = msg
                .get("tool_call_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !ref_id.is_empty() && !known_ids.contains(ref_id) {
                warn!(
                    tool_call_id = ref_id,
                    "[MessageSanitizer] Dropping orphaned tool result (tool_call_id not in known ids)"
                );
                continue;
            }
        }
        cleaned.push(msg);
    }
    cleaned
}

/// Compress a turn to first user text + last assistant text (`compress_turn_to_text_only`).
pub fn compress_turn_to_text_only(turn: &Value) -> Value {
    let messages = turn
        .get("messages")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();

    let mut user_text = String::new();
    let mut last_assistant_text = String::new();

    for msg in &messages {
        let role = role_str(msg).unwrap_or("");
        let content = msg.get("content").cloned().unwrap_or(Value::Null);

        if role == "user" {
            if let Some(blocks) = content.as_array() {
                if has_block_type(blocks, "tool_result") {
                    continue;
                }
            }
            if user_text.is_empty() {
                user_text = extract_text_from_content(&content);
            }
        } else if role == "assistant" {
            let text = extract_text_from_content(&content);
            if !text.is_empty() {
                last_assistant_text = text;
            }
        }
    }

    let mut compressed_messages = Vec::new();
    if !user_text.is_empty() {
        compressed_messages.push(json!({
            "role": "user",
            "content": [{"type": "text", "text": user_text}],
        }));
    }
    if !last_assistant_text.is_empty() {
        compressed_messages.push(json!({
            "role": "assistant",
            "content": [{"type": "text", "text": last_assistant_text}],
        }));
    }

    json!({ "messages": compressed_messages })
}

fn collect_tool_ids(messages: &[Value]) -> (HashSet<String>, HashSet<String>) {
    let mut use_ids = HashSet::new();
    let mut result_ids = HashSet::new();
    for msg in messages {
        let blocks = msg.get("content").and_then(|c| c.as_array());
        let Some(blocks) = blocks else {
            continue;
        };
        for block in blocks {
            if block_type(block) == Some("tool_use") {
                if let Some(id) = block_id(block) {
                    use_ids.insert(id.to_string());
                }
            } else if block_type(block) == Some("tool_result") {
                if let Some(id) = block_tool_use_id(block) {
                    result_ids.insert(id.to_string());
                }
            }
        }
    }
    (use_ids, result_ids)
}

fn has_block_type(content: &[Value], block_type_name: &str) -> bool {
    content
        .iter()
        .any(|b| block_type(b) == Some(block_type_name))
}

fn extract_text_from_content(content: &Value) -> String {
    if let Some(s) = content.as_str() {
        return s.trim().to_string();
    }
    if let Some(blocks) = content.as_array() {
        let parts: Vec<&str> = blocks
            .iter()
            .filter_map(|b| {
                if block_type(b) == Some("text") {
                    b.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .filter(|p| !p.is_empty())
            .collect();
        return parts.join("\n").trim().to_string();
    }
    String::new()
}

fn role_str(msg: &Value) -> Option<&str> {
    msg.get("role").and_then(|v| v.as_str())
}

fn content_blocks(msg: &Value) -> Option<&[Value]> {
    msg.get("content")
        .and_then(|c| c.as_array())
        .map(|a| a.as_slice())
}

fn block_type(block: &Value) -> Option<&str> {
    block.get("type").and_then(|t| t.as_str())
}

fn block_id(block: &Value) -> Option<&str> {
    block
        .get("id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

fn block_tool_use_id(block: &Value) -> Option<&str> {
    block
        .get("tool_use_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_appends_trailing_tool_results() {
        let mut messages = vec![json!({
            "role": "assistant",
            "content": [{"type": "tool_use", "id": "t1", "name": "x", "input": {}}]
        })];
        let n = repair_tool_use_adjacency(&mut messages);
        assert_eq!(n, 1);
        assert_eq!(messages.len(), 2);
        assert_eq!(role_str(&messages[1]), Some("user"));
        let blocks = messages[1]["content"].as_array().unwrap();
        assert_eq!(blocks[0]["tool_use_id"], "t1");
        assert_eq!(blocks[0]["is_error"], true);
    }

    #[test]
    fn repair_inserts_when_next_not_user() {
        let mut messages = vec![
            json!({
                "role": "assistant",
                "content": [{"type": "tool_use", "id": "t1", "name": "x", "input": {}}]
            }),
            json!({"role": "assistant", "content": [{"type": "text", "text": "oops"}]}),
        ];
        let n = repair_tool_use_adjacency(&mut messages);
        assert_eq!(n, 1);
        assert_eq!(role_str(&messages[1]), Some("user"));
        assert_eq!(role_str(&messages[2]), Some("assistant"));
    }

    #[test]
    fn repair_prepends_missing_tool_results() {
        let mut messages = vec![
            json!({
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "t1", "name": "a", "input": {}},
                    {"type": "tool_use", "id": "t2", "name": "b", "input": {}},
                ]
            }),
            json!({
                "role": "user",
                "content": [{"type": "tool_result", "tool_use_id": "t1", "content": "ok"}]
            }),
        ];
        let n = repair_tool_use_adjacency(&mut messages);
        assert_eq!(n, 1);
        let blocks = messages[1]["content"].as_array().unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0]["tool_use_id"], "t2");
        assert!(blocks[0]["is_error"].as_bool().unwrap());
    }

    #[test]
    fn sanitize_removes_leading_orphan_tool_result() {
        let mut messages = vec![
            json!({
                "role": "user",
                "content": [{"type": "tool_result", "tool_use_id": "ghost", "content": "x"}]
            }),
            json!({
                "role": "user",
                "content": [{"type": "text", "text": "hi"}]
            }),
        ];
        let n = sanitize_claude_messages(&mut messages);
        assert!(n >= 1);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["content"][0]["type"], "text");
    }

    #[test]
    fn sanitize_repairs_adjacency_on_user_text_only() {
        // Python runs adjacency repair first: prepends synthetic tool_result onto next user msg.
        let mut messages = vec![
            json!({
                "role": "assistant",
                "content": [{"type": "tool_use", "id": "orphan", "name": "x", "input": {}}]
            }),
            json!({
                "role": "user",
                "content": [{"type": "text", "text": "no result"}]
            }),
        ];
        let n = sanitize_claude_messages(&mut messages);
        assert!(n >= 1);
        assert_eq!(messages.len(), 2);
        let blocks = messages[1]["content"].as_array().unwrap();
        assert!(blocks
            .iter()
            .any(|b| b.get("tool_use_id") == Some(&json!("orphan"))));
        assert!(blocks.iter().any(|b| b.get("type") == Some(&json!("text"))));
    }

    #[test]
    fn sanitize_removes_truly_unmatched_tool_use() {
        // tool_use with no following user message at all — adjacency appends user; then stable.
        let mut messages = vec![json!({
            "role": "assistant",
            "content": [{"type": "tool_use", "id": "only", "name": "x", "input": {}}]
        })];
        let n = sanitize_claude_messages(&mut messages);
        assert!(n >= 1);
        assert_eq!(messages.len(), 2);
        assert_eq!(role_str(&messages[1]), Some("user"));
    }

    #[test]
    fn sanitize_strips_bad_tool_result_keeps_text() {
        let mut messages = vec![json!({
            "role": "user",
            "content": [
                {"type": "tool_result", "tool_use_id": "bad", "content": "err"},
                {"type": "text", "text": "keep me"},
            ]
        })];
        sanitize_claude_messages(&mut messages);
        let blocks = messages[0]["content"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "text");
    }

    #[test]
    fn drop_orphaned_openai_tool_messages() {
        let messages = vec![
            json!({
                "role": "assistant",
                "tool_calls": [{"id": "call_a", "type": "function", "function": {"name": "f", "arguments": "{}"}}]
            }),
            json!({"role": "tool", "tool_call_id": "call_a", "content": "ok"}),
            json!({"role": "tool", "tool_call_id": "call_missing", "content": "drop"}),
        ];
        let cleaned = drop_orphaned_tool_results_openai(messages);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaned[1]["tool_call_id"], "call_a");
    }

    #[test]
    fn compress_turn_keeps_first_user_and_last_assistant_text() {
        let turn = json!({
            "messages": [
                {"role": "user", "content": [{"type": "text", "text": "question"}]},
                {"role": "assistant", "content": [{"type": "tool_use", "id": "1", "name": "t", "input": {}}]},
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "1", "content": "data"}]},
                {"role": "assistant", "content": [{"type": "text", "text": "draft"}]},
                {"role": "assistant", "content": [{"type": "text", "text": "final answer"}]},
            ]
        });
        let out = compress_turn_to_text_only(&turn);
        let msgs = out["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["content"][0]["text"], "question");
        assert_eq!(msgs[1]["content"][0]["text"], "final answer");
    }

    #[test]
    fn extract_text_from_string_content() {
        assert_eq!(extract_text_from_content(&json!("  hello  ")), "hello");
    }
}
