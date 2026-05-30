//! Token estimation (`agent/protocol/agent.py`).

use serde_json::Value;

/// Model context window by name (`Agent._get_model_context_window`).
pub fn model_context_window(model_name: &str) -> u32 {
    let model_name = model_name.to_lowercase();

    if model_name.contains("claude-3") || model_name.contains("claude-sonnet") {
        return 200_000;
    }
    if model_name.contains("gpt-4") {
        if model_name.contains("turbo") || model_name.contains("128k") {
            return 128_000;
        }
        if model_name.contains("32k") {
            return 32_000;
        }
        return 8_000;
    }
    if model_name.contains("gpt-3.5") {
        if model_name.contains("16k") {
            return 16_000;
        }
        return 4_000;
    }
    if model_name.contains("deepseek") {
        return 64_000;
    }
    if model_name.contains("gemini") {
        if model_name.contains("2.0") || model_name.contains("exp") {
            return 2_000_000;
        }
        return 1_000_000;
    }
    128_000
}

pub fn context_reserve_tokens(context_window: u32, explicit: Option<u32>) -> u32 {
    if let Some(r) = explicit {
        return r;
    }
    let reserve = (context_window as f64 * 0.1) as u32;
    reserve.clamp(10_000, 200_000)
}

pub fn estimate_text_tokens(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    let non_ascii = text.chars().filter(|c| (*c as u32) > 127).count() as u32;
    let ascii_count = text.len() as u32 - non_ascii;
    (non_ascii as f64 * 1.5 + ascii_count as f64 * 0.25) as u32 + 1
}

/// `Agent._estimate_message_tokens`
pub fn estimate_message_tokens(message: &Value) -> u32 {
    let content = message.get("content").cloned().unwrap_or(Value::Null);
    if let Some(s) = content.as_str() {
        return estimate_text_tokens(s).max(1);
    }
    if let Some(blocks) = content.as_array() {
        let mut total = 0u32;
        for part in blocks {
            let Some(obj) = part.as_object() else {
                continue;
            };
            let block_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match block_type {
                "text" => {
                    total += estimate_text_tokens(
                        obj.get("text").and_then(|t| t.as_str()).unwrap_or(""),
                    );
                }
                "image" => total += 1200,
                "tool_use" => {
                    total += 50;
                    if let Some(input) = obj.get("input") {
                        let input_str = serde_json::to_string(input).unwrap_or_default();
                        total += estimate_text_tokens(&input_str);
                    }
                }
                "tool_result" => {
                    total += 30;
                    if let Some(s) = obj.get("content").and_then(|c| c.as_str()) {
                        total += estimate_text_tokens(s);
                    }
                }
                _ => total += 10,
            }
        }
        return total.max(1);
    }
    1
}

pub fn estimate_turn_tokens(turn_messages: &[Value]) -> u32 {
    turn_messages.iter().map(estimate_message_tokens).sum()
}
