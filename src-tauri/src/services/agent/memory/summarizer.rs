//! Conversation auto-summary pipeline — compress long sessions into memory markdown.

use std::path::Path;

use serde_json::Value;
use tracing::{info, warn};

use super::config::MemoryConfig;
use super::conversation_store::ConversationStore;
use crate::config::ModelsConfig;
use crate::services::agent::rig::run_simple_chat;

/// Check session length and optionally write a summary markdown file.
pub async fn maybe_summarize_conversation(
    workspace: &Path,
    config: &ModelsConfig,
    mem_config: &MemoryConfig,
    store: &ConversationStore,
    session_id: &str,
    channel_type: &str,
) {
    if !mem_config.auto_conversation_summary {
        return;
    }
    let threshold = mem_config.summary_message_threshold.max(10);
    let msg_count = match store.session_msg_count(session_id) {
        Ok(c) => c,
        Err(e) => {
            warn!("[Summarizer] msg_count failed session={session_id}: {e}");
            return;
        }
    };
    if msg_count < threshold as i64 {
        return;
    }
    let checkpoint = store.last_summary_checkpoint(session_id).unwrap_or(0);
    if msg_count < checkpoint + threshold as i64 {
        return;
    }

    let messages = match store.load_messages(session_id, u32::MAX) {
        Ok(m) => m,
        Err(e) => {
            warn!("[Summarizer] load failed session={session_id}: {e}");
            return;
        }
    };
    if messages.is_empty() {
        return;
    }

    let transcript = format_transcript(&messages);
    if transcript.trim().is_empty() {
        return;
    }

    let prompt = format!(
        "请将以下对话压缩为结构化摘要（中文），保留关键事实、用户偏好与待办。\
         输出纯 Markdown，不要代码块包裹。\n\n{transcript}"
    );

    let summary = match run_simple_chat(config, &prompt).await {
        Ok(s) => s,
        Err(e) => {
            warn!("[Summarizer] LLM failed session={session_id}: {e}");
            return;
        }
    };

    if let Err(e) = write_summary_file(workspace, session_id, channel_type, msg_count, &summary) {
        warn!("[Summarizer] write failed session={session_id}: {e}");
        return;
    }

    if let Err(e) = store.set_last_summary_checkpoint(session_id, msg_count) {
        warn!("[Summarizer] checkpoint failed session={session_id}: {e}");
    }

    info!(
        session_id,
        msg_count, "[Summarizer] wrote conversation summary to memory/"
    );
}

fn format_transcript(messages: &[Value]) -> String {
    let mut lines = Vec::new();
    for msg in messages.iter().take(80) {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("?");
        let text = extract_text(msg.get("content").unwrap_or(&Value::Null));
        if text.is_empty() {
            continue;
        }
        let clipped: String = text.chars().take(800).collect();
        lines.push(format!("{role}: {clipped}"));
    }
    lines.join("\n")
}

fn extract_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| {
                if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                    b.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn write_summary_file(
    workspace: &Path,
    session_id: &str,
    channel_type: &str,
    msg_count: i64,
    summary: &str,
) -> Result<(), String> {
    let memory_dir = workspace.join("memory");
    std::fs::create_dir_all(&memory_dir).map_err(|e| e.to_string())?;
    let safe_id: String = session_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let path = memory_dir.join(format!("summary-{safe_id}-{msg_count}.md"));
    let body = format!(
        "---\nsource: conversation_summary\nsession_id: {session_id}\nchannel: {channel_type}\nmsg_count: {msg_count}\n---\n\n{summary}\n"
    );
    std::fs::write(&path, body).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_frontmatter_marks_source() {
        let dir = std::env::temp_dir().join(format!("sf-summary-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        write_summary_file(&dir, "sess-1", "wework", 42, "用户偏好简洁回复。").unwrap();
        let path = dir.join("memory/summary-sess-1-42.md");
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.contains("source: conversation_summary"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
