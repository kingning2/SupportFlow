//! Token counting helpers from `models/**/**_session.py` and `chat_gpt_session.py`.

use serde_json::Value;
use tiktoken_rs::{cl100k_base, o200k_base, CoreBPE};
use tracing::debug;

/// `deepseek_session.num_tokens_from_messages` — string or list text blocks.
pub fn num_tokens_content_blocks(messages: &[Value], _model: &str) -> u32 {
    let mut tokens = 0u32;
    for msg in messages {
        match msg.get("content") {
            Some(Value::String(s)) => tokens += s.chars().count() as u32,
            Some(Value::Array(blocks)) => {
                for block in blocks {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        tokens += text.chars().count() as u32;
                    }
                }
            }
            _ => {}
        }
    }
    tokens
}

/// `doubao_session` / `moonshot_session` — `len(content)` for string content only.
pub fn num_tokens_len_content(messages: &[Value], _model: &str) -> u32 {
    let mut tokens = 0u32;
    for msg in messages {
        if let Some(Value::String(s)) = msg.get("content") {
            tokens += s.chars().count() as u32;
        }
    }
    tokens
}

/// `dashscope_session.num_tokens_from_messages`
pub fn num_tokens_dashscope(messages: &[Value]) -> u32 {
    let mut tokens = 0u32;
    for msg in messages {
        if let Some(Value::String(s)) = msg.get("content") {
            tokens += s.chars().count() as u32;
        }
    }
    tokens
}

/// `baidu_wenxin_session.num_tokens_from_messages`
pub fn num_tokens_baidu_wenxin(messages: &[Value], _model: &str) -> u32 {
    num_tokens_len_content(messages, _model)
}

/// `minimax_session.num_tokens_from_messages`
pub fn num_tokens_minimax(messages: &[Value], _model: &str) -> u32 {
    let mut tokens = 0u32;
    for msg in messages {
        if let Some(Value::String(s)) = msg.get("text") {
            tokens += s.chars().count() as u32;
        }
    }
    tokens
}

/// `chat_gpt_session.num_tokens_by_character`
pub fn num_tokens_by_character(messages: &[Value]) -> u32 {
    let mut tokens = 0u32;
    for msg in messages {
        if let Some(Value::String(s)) = msg.get("content") {
            tokens += s.chars().count() as u32;
        }
    }
    tokens
}

fn encoding_for_model(model: &str) -> CoreBPE {
    if model.starts_with("o1") || model.starts_with("gpt-5") {
        o200k_base().unwrap_or_else(|_| cl100k_base().expect("cl100k_base"))
    } else {
        cl100k_base().unwrap_or_else(|_| cl100k_base().expect("cl100k_base"))
    }
}

/// `open_ai_session.num_tokens_from_string`
pub fn num_tokens_from_string(string: &str, model: &str) -> u32 {
    let bpe = encoding_for_model(model);
    bpe.encode_with_special_tokens(string).len() as u32
}

/// `chat_gpt_session.num_tokens_from_messages` (tiktoken rules).
pub fn num_tokens_from_messages_chatgpt(messages: &[Value], model: &str) -> u32 {
    if model == "wenxin" || model == "xunfei" || model.starts_with("gemini") {
        return num_tokens_by_character(messages);
    }

    let effective = chatgpt_effective_model(model);
    count_chatgpt_messages(messages, &effective)
}

fn chatgpt_effective_model(model: &str) -> String {
    if matches!(
        model,
        "gpt-3.5-turbo-0301" | "gpt-35-turbo" | "gpt-3.5-turbo-1106" | "moonshot"
    ) || model == "linkai-3.5"
    {
        return "gpt-3.5-turbo".to_string();
    }
    if matches!(
        model,
        "gpt-4-0314"
            | "gpt-4-0613"
            | "gpt-4-32k"
            | "gpt-4-32k-0613"
            | "gpt-3.5-turbo-0613"
            | "gpt-3.5-turbo-16k"
            | "gpt-3.5-turbo-16k-0613"
            | "gpt-35-turbo-16k"
            | "gpt-4-turbo-preview"
            | "gpt-4-1106-preview"
            | "gpt-4-vision-preview"
            | "gpt-4-turbo-2024-01-25"
            | "gpt-4o"
            | "gpt-4o-2024-08-06"
            | "gpt-4o-mini"
            | "gpt-5"
            | "gpt-5-mini"
            | "gpt-5-nano"
    ) {
        return "gpt-4".to_string();
    }
    if model.starts_with("claude-3") {
        return "gpt-3.5-turbo".to_string();
    }
    model.to_string()
}

fn count_chatgpt_messages(messages: &[Value], model: &str) -> u32 {
    let bpe = encoding_for_model(model);
    let (tokens_per_message, tokens_per_name) = if model == "gpt-3.5-turbo" {
        (4u32, -1i32)
    } else if model == "gpt-4" {
        (3u32, 1i32)
    } else {
        debug!(
            model,
            "num_tokens_from_messages() not implemented; assuming gpt-3.5-turbo"
        );
        return count_chatgpt_messages(messages, "gpt-3.5-turbo");
    };

    let mut num_tokens = 0u32;
    for message in messages {
        num_tokens += tokens_per_message;
        if let Value::Object(map) = message {
            for (key, value) in map {
                let s = value_as_token_string(value);
                num_tokens += bpe.encode_with_special_tokens(&s).len() as u32;
                if key == "name" {
                    num_tokens = (num_tokens as i32 + tokens_per_name).max(0) as u32;
                }
            }
        }
    }
    num_tokens + 3
}

fn value_as_token_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
