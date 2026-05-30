//! Error types mirroring `models/openai/openai_compat.py` (subset).

use thiserror::Error;

#[derive(Debug, Error)]
pub struct OpenAiHttpError {
    pub status_code: u16,
    pub body: serde_json::Value,
    pub message: String,
}

impl std::fmt::Display for OpenAiHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP {}: {}", self.status_code, self.message)
    }
}

impl OpenAiHttpError {
    pub fn from_response(status: u16, body: serde_json::Value) -> Self {
        let message = extract_error_message(&body).unwrap_or_else(|| body.to_string());
        Self {
            status_code: status,
            body,
            message,
        }
    }
}

fn extract_error_message(body: &serde_json::Value) -> Option<String> {
    let err = body.get("error")?;
    match err {
        serde_json::Value::Object(o) => o
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        serde_json::Value::String(s) => Some(s.clone()),
        _ => None,
    }
}
