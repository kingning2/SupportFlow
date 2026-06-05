//! Top-level helpers from `agent_stream.py`.

use serde_json::Value;

pub const MAX_STORED_REASONING_CHARS: usize = 4 * 1024;

const REASONING_TRUNCATE_MARKER: &str =
    "\n\n... [reasoning truncated, {omitted} chars omitted] ...\n\n";

/// Trim long reasoning for DB / history storage.
pub fn truncate_reasoning_for_storage(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    if text.len() <= MAX_STORED_REASONING_CHARS {
        return text.to_string();
    }
    let half = MAX_STORED_REASONING_CHARS / 2;
    let head = &text[..half];
    let tail = &text[text.len() - half..];
    let omitted = text.len() - head.len() - tail.len();
    format!(
        "{head}{}{tail}",
        REASONING_TRUNCATE_MARKER.replace("{omitted}", &omitted.to_string())
    )
}

/// Parse tool args JSON. Returns `(args, error_msg)`; `error_msg` is `None` on success.
pub fn parse_tool_args(args_str: &str, finish_reason: Option<&str>) -> (Value, Option<String>) {
    if args_str.is_empty() {
        return (Value::Object(serde_json::Map::new()), None);
    }
    match serde_json::from_str::<Value>(args_str) {
        Ok(Value::Object(map)) => (Value::Object(map), None),
        Ok(_) => (Value::Object(serde_json::Map::new()), None),
        Err(e) => {
            if matches!(finish_reason, Some("length") | Some("max_tokens"))
                || !args_str.trim_end().ends_with('}')
            {
                return (
                    Value::Object(serde_json::Map::new()),
                    Some(
                        "Output truncated (max_tokens reached). Split content into smaller chunks across multiple tool calls.".into(),
                    ),
                );
            }
            (
                Value::Object(serde_json::Map::new()),
                Some(format!("Invalid JSON in tool arguments: {}", e)),
            )
        }
    }
}

/// Strip or keep `<think>` blocks per channel policy.
pub fn filter_think_tags(text: &str, render_thinking_inline: bool) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    if render_thinking_inline {
        return text.replace("<think>", "").replace("</think>", "");
    }
    let open = "<think>";
    let close = "</think>";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        let Some(start) = rest.find(open) else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..start]);
        let after_open = &rest[start + open.len()..];
        if let Some(end_rel) = after_open.find(close) {
            rest = &after_open[end_rel + close.len()..];
        } else {
            break;
        }
    }
    out
}

pub fn hash_args(args: &Value) -> String {
    let args_str = serde_json::to_string(args).unwrap_or_else(|_| "{}".into());
    format!("{:x}", md5::compute(args_str.as_bytes()))
        .chars()
        .take(8)
        .collect()
}

pub fn is_context_overflow_error(error_str: &str) -> bool {
    let lower = error_str.to_lowercase();
    if lower.contains("[context_overflow]") {
        return true;
    }
    [
        "context length exceeded",
        "maximum context length",
        "prompt is too long",
        "context overflow",
        "context window",
        "too large",
        "exceeds model context",
        "request_too_large",
        "request exceeds the maximum size",
        "tokens exceed",
    ]
    .iter()
    .any(|k| lower.contains(k))
}

pub fn is_message_format_error(error_str: &str) -> bool {
    let lower = error_str.to_lowercase();
    let keyword_hit = [
        "tool_use",
        "tool_result",
        "tool result",
        "without",
        "immediately after",
        "corresponding",
        "must have",
        "each",
        "tool_call_id",
        "tool id",
        "is not found",
        "not found",
        "tool_calls",
        "must be a response to a preceeding message",
        "2013",
    ]
    .iter()
    .any(|k| lower.contains(k));

    keyword_hit
        && (lower.contains("400")
            || lower.contains("status: 400")
            || lower.contains("invalid_request")
            || lower.contains("invalidparameter"))
}

pub fn is_retryable_llm_error(error_str: &str) -> bool {
    let lower = error_str.to_lowercase();
    [
        "timeout",
        "timed out",
        "connection",
        "network",
        "rate limit",
        "overloaded",
        "unavailable",
        "busy",
        "retry",
        "429",
        "500",
        "502",
        "503",
        "504",
        "512",
    ]
    .iter()
    .any(|k| lower.contains(k))
}

pub fn is_rate_limit_error(error_str: &str) -> bool {
    let lower = error_str.to_lowercase();
    lower.contains("429") || lower.contains("rate limit")
}

pub fn extract_stream_error(chunk: &Value) -> (String, String, String, u16) {
    let error_data = chunk.get("error");
    let (error_msg, error_code, error_type) = if let Some(Value::Object(o)) = error_data {
        (
            o.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string(),
            o.get("code")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
            o.get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string(),
        )
    } else if let Some(s) = error_data.and_then(|e| e.as_str()) {
        (s.to_string(), String::new(), String::new())
    } else {
        (
            chunk
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string(),
            String::new(),
            String::new(),
        )
    };
    let status_code = chunk
        .get("status_code")
        .and_then(|v| v.as_u64())
        .unwrap_or(500) as u16;
    (error_msg, error_code, error_type, status_code)
}

pub fn chunk_is_top_level_error(chunk: &Value) -> bool {
    if chunk.get("error") == Some(&Value::Bool(true)) {
        return true;
    }
    chunk.get("error").is_some() && chunk.get("choices").is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_reasoning_splits_head_tail() {
        let text = "a".repeat(10_000);
        let out = truncate_reasoning_for_storage(&text);
        assert!(out.len() < text.len());
        assert!(out.contains("reasoning truncated"));
    }

    #[test]
    fn parse_tool_args_detects_truncation() {
        let (v, err) = parse_tool_args(r#"{"x": "#, Some("length"));
        assert!(v.is_object());
        assert!(err.is_some());
    }
}
