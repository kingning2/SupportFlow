//! `models/openai_compatible_bot.py` — shared tool-calling for OpenAI-compatible APIs.

use std::pin::Pin;

use async_trait::async_trait;
use futures_util::Stream;
use serde_json::{json, Map, Value};
use tracing::error;

use crate::message_utils::convert_messages_to_openai_format;
use crate::openai::{OpenAiHttpClient, OpenAiHttpError};

/// API settings returned by `get_api_config()` (Python dict shape).
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    pub default_temperature: f32,
    pub default_top_p: f32,
    pub default_frequency_penalty: f32,
    pub default_presence_penalty: f32,
}

/// Agent / bridge request for `call_with_tools` (`**kwargs` subset).
#[derive(Debug, Clone, Default)]
pub struct CallWithToolsRequest {
    pub messages: Vec<Value>,
    pub tools: Option<Vec<Value>>,
    pub stream: bool,
    pub model: Option<String>,
    pub max_tokens: Option<u64>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub system: Option<String>,
    pub tool_choice: Option<Value>,
    pub request_timeout: Option<u64>,
    pub timeout: Option<u64>,
    pub extra: Map<String, Value>,
}

/// Sync JSON body or SSE chunk stream (agent protocol chunks).
pub enum LlmResult {
    Complete(Value),
    Stream(Pin<Box<dyn Stream<Item = Value> + Send>>),
}

#[async_trait]
pub trait OpenAICompatibleBot: Send + Sync {
    fn get_api_config(&self) -> ApiConfig;
    fn http_client(&self) -> &OpenAiHttpClient;

    async fn call_with_tools(
        &self,
        req: CallWithToolsRequest,
    ) -> Result<LlmResult, OpenAiHttpError> {
        call_with_tools_impl(self, req).await
    }
}

/// Back-compat alias; prefer [`OpenAICompatibleBot::call_with_tools`].
pub trait OpenAICompatibleBotExt: OpenAICompatibleBot {}
impl<T: OpenAICompatibleBot + ?Sized> OpenAICompatibleBotExt for T {}

pub async fn call_with_tools_impl<B: OpenAICompatibleBot + ?Sized>(
    bot: &B,
    req: CallWithToolsRequest,
) -> Result<LlmResult, OpenAiHttpError> {
    let api = bot.get_api_config();
    let client = bot.http_client();

    let mut messages = convert_messages_to_openai_format(req.messages);

    if let Some(system) = req.system {
        if messages
            .first()
            .and_then(|m| m.get("role"))
            .and_then(|r| r.as_str())
            != Some("system")
        {
            messages.insert(0, json!({ "role": "system", "content": system }));
        } else if let Some(Value::Object(ref mut map)) = messages.first_mut() {
            map.insert("content".into(), json!(system));
        }
    }

    let tools = req.tools.map(convert_tools_to_openai);

    let model_name = req.model.unwrap_or(api.model);
    let mut payload = Map::new();
    payload.insert("model".into(), json!(model_name));
    payload.insert("messages".into(), Value::Array(messages));
    payload.insert(
        "temperature".into(),
        json!(req.temperature.unwrap_or(api.default_temperature)),
    );
    payload.insert(
        "top_p".into(),
        json!(req.top_p.unwrap_or(api.default_top_p)),
    );
    payload.insert(
        "frequency_penalty".into(),
        json!(req
            .frequency_penalty
            .unwrap_or(api.default_frequency_penalty)),
    );
    payload.insert(
        "presence_penalty".into(),
        json!(req.presence_penalty.unwrap_or(api.default_presence_penalty)),
    );
    payload.insert("stream".into(), json!(req.stream));

    if is_restricted_sampling_model(&model_name) {
        payload.remove("temperature");
        payload.remove("top_p");
        payload.remove("frequency_penalty");
        payload.remove("presence_penalty");
    }

    if let Some(max_tokens) = req.max_tokens {
        payload.insert("max_tokens".into(), json!(max_tokens));
    }
    if let Some(tools) = tools {
        payload.insert("tools".into(), Value::Array(tools));
        payload.insert(
            "tool_choice".into(),
            req.tool_choice.unwrap_or(json!("auto")),
        );
    }
    for (k, v) in req.extra {
        payload.insert(k, v);
    }

    let timeout_secs = req
        .request_timeout
        .or(req.timeout)
        .or(Some(api_default_timeout()));

    let api_key = if api.api_key.is_empty() {
        None
    } else {
        Some(api.api_key.as_str())
    };
    let api_base = if api.api_base.is_empty() {
        None
    } else {
        Some(api.api_base.as_str())
    };

    if req.stream {
        match client
            .chat_completions_stream(payload, api_key, api_base, timeout_secs)
            .await
        {
            Ok(stream) => {
                let s: Pin<Box<dyn Stream<Item = Value> + Send>> = Box::pin(stream);
                Ok(LlmResult::Stream(s))
            }
            Err(e) => Ok(LlmResult::Complete(error_value(&e))),
        }
    } else {
        match client
            .chat_completions(payload, api_key, api_base, timeout_secs)
            .await
        {
            Ok(body) => Ok(LlmResult::Complete(body)),
            Err(e) => Ok(LlmResult::Complete(error_value(&e))),
        }
    }
}

fn api_default_timeout() -> u64 {
    600
}

fn is_restricted_sampling_model(model: &str) -> bool {
    matches!(
        model,
        "gpt-5" | "gpt-5-mini" | "gpt-5-nano" | "gpt-5.5" | "o1" | "o1-mini"
    )
}

fn convert_tools_to_openai(tools: Vec<Value>) -> Vec<Value> {
    let mut out = Vec::with_capacity(tools.len());
    for tool in tools {
        if tool.get("type").and_then(|t| t.as_str()) == Some("function") {
            out.push(tool);
            continue;
        }
        out.push(json!({
            "type": "function",
            "function": {
                "name": tool.get("name").cloned().unwrap_or(Value::Null),
                "description": tool.get("description").cloned().unwrap_or(Value::Null),
                "parameters": tool.get("input_schema").cloned().unwrap_or_else(|| json!({})),
            }
        }));
    }
    out
}

pub fn error_value(err: &OpenAiHttpError) -> Value {
    error!(status = err.status_code, msg = %err.message, "call_with_tools error");
    json!({
        "error": true,
        "message": err.message,
        "status_code": if err.status_code == 0 { 500 } else { err.status_code }
    })
}
