//! Agent-mode stream/sync transforms (`deepseek_bot._handle_stream_response` / `_handle_sync_response`).

use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::stream;
use futures_util::Stream;
use serde_json::{json, Value};

use crate::openai::openai_http_client::SseChunkStream;

/// Wraps raw OpenAI SSE and yields agent-facing chunks (OpenAI delta shape + final `finish_reason`).
pub struct DeepSeekAgentStream {
    inner: Pin<Box<dyn Stream<Item = Value> + Send>>,
    pending: VecDeque<Value>,
    finish_reason: Option<Value>,
    emitted_terminal: bool,
}

impl DeepSeekAgentStream {
    pub fn new(inner: SseChunkStream) -> Self {
        Self::wrap(inner)
    }

    fn wrap(inner: impl Stream<Item = Value> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(inner),
            pending: VecDeque::new(),
            finish_reason: None,
            emitted_terminal: false,
        }
    }

    /// Feed pre-parsed SSE JSON objects (unit/integration tests; not wire-format SSE).
    pub fn from_raw_chunks(chunks: Vec<Value>) -> Pin<Box<Self>> {
        Box::pin(Self::wrap(stream::iter(chunks)))
    }

    pub fn from_error(message: impl Into<String>, status_code: u16) -> Pin<Box<Self>> {
        Box::pin(Self {
            inner: Box::pin(stream::empty()),
            pending: VecDeque::from([agent_error_chunk(message, status_code)]),
            finish_reason: None,
            emitted_terminal: true,
        })
    }

    fn push_terminal(&mut self) {
        if self.emitted_terminal {
            return;
        }
        self.emitted_terminal = true;
        self.pending.push_back(json!({
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": self.finish_reason.clone(),
            }]
        }));
    }
}

impl Stream for DeepSeekAgentStream {
    type Item = Value;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(item) = self.pending.pop_front() {
                return Poll::Ready(Some(item));
            }

            if self.emitted_terminal {
                return Poll::Ready(None);
            }

            match self.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(raw)) => {
                    if let Some(err) = extract_top_level_error(&raw) {
                        self.pending.push_back(err);
                        self.emitted_terminal = true;
                        continue;
                    }

                    if let Some(fr) = extract_finish_reason(&raw) {
                        self.finish_reason = Some(json!(fr));
                    }

                    for chunk in transform_raw_sse_chunk(&raw) {
                        self.pending.push_back(chunk);
                    }
                }
                Poll::Ready(None) => {
                    self.push_terminal();
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Single-shot OpenAI JSON → Claude-style assistant message (`_handle_sync_response`).
pub fn transform_sync_response(result: &Value) -> Value {
    let choice = result.get("choices").and_then(|c| c.get(0));
    let message = choice.and_then(|c| c.get("message"));
    let finish_reason = choice
        .and_then(|c| c.get("finish_reason"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut content_blocks = Vec::new();

    if let Some(reasoning) = message
        .and_then(|m| m.get("reasoning_content"))
        .and_then(|r| r.as_str())
    {
        if !reasoning.is_empty() {
            content_blocks.push(json!({
                "type": "thinking",
                "thinking": reasoning,
            }));
        }
    }

    if let Some(text) = message
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
    {
        if !text.is_empty() {
            content_blocks.push(json!({
                "type": "text",
                "text": text,
            }));
        }
    }

    if let Some(tool_calls) = message
        .and_then(|m| m.get("tool_calls"))
        .and_then(|t| t.as_array())
    {
        for tool_call in tool_calls {
            let args_str = tool_call
                .get("function")
                .and_then(|f| f.get("arguments"))
                .and_then(|a| a.as_str())
                .unwrap_or("{}");
            let tool_input: Value = serde_json::from_str(args_str).unwrap_or_else(|_| json!({}));
            content_blocks.push(json!({
                "type": "tool_use",
                "id": tool_call.get("id").cloned().unwrap_or(Value::Null),
                "name": tool_call
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .cloned()
                    .unwrap_or(Value::Null),
                "input": tool_input,
            }));
        }
    }

    let stop_reason = match finish_reason {
        "tool_calls" => "tool_use",
        "stop" => "end_turn",
        other => other,
    };

    json!({
        "role": "assistant",
        "content": content_blocks,
        "stop_reason": stop_reason,
    })
}

pub fn agent_error_chunk(message: impl Into<String>, status_code: u16) -> Value {
    json!({
        "error": true,
        "message": message.into(),
        "status_code": if status_code == 0 { 500 } else { status_code },
    })
}

fn extract_top_level_error(chunk: &Value) -> Option<Value> {
    if chunk.get("error") == Some(&Value::Bool(true)) {
        return Some(chunk.clone());
    }
    if let Some(err) = chunk.get("error") {
        if err.is_object() || err.is_string() {
            let msg = if let Some(o) = err.as_object() {
                o.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string()
            } else {
                err.as_str().unwrap_or("Unknown error").to_string()
            };
            return Some(agent_error_chunk(msg, 500));
        }
    }
    None
}

fn extract_finish_reason(chunk: &Value) -> Option<String> {
    chunk
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("finish_reason"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// Map one raw SSE JSON object to zero or more agent chunks.
#[doc(hidden)]
pub fn transform_raw_sse_chunk(chunk: &Value) -> Vec<Value> {
    let mut out = Vec::new();

    if chunk.get("choices").is_none() {
        return out;
    }

    let choice = match chunk.get("choices").and_then(|c| c.get(0)) {
        Some(c) => c,
        None => return out,
    };
    let delta = choice.get("delta").cloned().unwrap_or(Value::Null);

    if let Some(reasoning) = delta.get("reasoning_content").and_then(|r| r.as_str()) {
        if !reasoning.is_empty() {
            out.push(json!({
                "choices": [{
                    "index": 0,
                    "delta": {
                        "role": "assistant",
                        "reasoning_content": reasoning,
                    },
                    "finish_reason": null,
                }]
            }));
        }
    }

    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
        if !content.is_empty() {
            out.push(json!({
                "choices": [{
                    "index": 0,
                    "delta": {
                        "role": "assistant",
                        "content": content,
                    },
                }]
            }));
        }
    }

    if let Some(tool_calls) = delta.get("tool_calls").and_then(|t| t.as_array()) {
        for tool_call_chunk in tool_calls {
            out.push(json!({
                "choices": [{
                    "index": 0,
                    "delta": { "tool_calls": [tool_call_chunk.clone()] },
                }]
            }));
        }
    }

    out
}
