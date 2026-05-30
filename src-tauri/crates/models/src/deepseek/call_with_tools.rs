//! `DeepSeekBot.call_with_tools` and agent-mode stream/sync handlers.

use serde_json::{json, Map, Value};
use tracing::debug;

use crate::message_utils::convert_messages_to_openai_format;
use crate::openai::OpenAiHttpError;
use crate::openai_compatible::{CallWithToolsRequest, LlmResult};

use super::agent_stream::{agent_error_chunk, transform_sync_response, DeepSeekAgentStream};
use super::deepseek_bot::DeepSeekBot;

impl DeepSeekBot {
    pub(crate) async fn call_with_tools_deepseek(
        &self,
        req: CallWithToolsRequest,
    ) -> Result<LlmResult, OpenAiHttpError> {
        let model = req.model.clone().unwrap_or_else(|| {
            self.config
                .model_or(crate::const_::BotType::DEEPSEEK_V4_FLASH)
        });

        let body = build_request_body(&req, &model);

        let api_key = if self.api_key().is_empty() {
            None
        } else {
            Some(self.api_key())
        };
        let api_base = self.api_base();
        let timeout = req.request_timeout.or(req.timeout).or(Some(180));

        if req.stream {
            match self
                .client
                .chat_completions_stream(body, api_key.as_deref(), Some(&api_base), timeout)
                .await
            {
                Ok(raw) => {
                    let agent = DeepSeekAgentStream::new(raw);
                    let s: std::pin::Pin<Box<dyn futures_util::Stream<Item = Value> + Send>> =
                        Box::pin(agent);
                    Ok(LlmResult::Stream(s))
                }
                Err(e) => Ok(stream_error_result(&e.message, e.status_code)),
            }
        } else {
            let mut body = body;
            body.remove("stream");
            match self
                .client
                .chat_completions(body, api_key.as_deref(), Some(&api_base), timeout)
                .await
            {
                Ok(v) => Ok(LlmResult::Complete(transform_sync_response(&v))),
                Err(e) => Ok(LlmResult::Complete(agent_error_chunk(
                    &e.message,
                    e.status_code,
                ))),
            }
        }
    }
}

fn stream_error_result(message: &str, status_code: u16) -> LlmResult {
    let s: std::pin::Pin<Box<dyn futures_util::Stream<Item = Value> + Send>> =
        DeepSeekAgentStream::from_error(message, status_code);
    LlmResult::Stream(s)
}

fn build_request_body(req: &CallWithToolsRequest, model: &str) -> Map<String, Value> {
    let mut converted = convert_messages_to_openai_format(req.messages.clone());
    if let Some(system) = &req.system {
        if converted
            .first()
            .and_then(|m| m.get("role"))
            .and_then(|r| r.as_str())
            != Some("system")
        {
            converted.insert(0, json!({ "role": "system", "content": system }));
        } else if let Some(Value::Object(ref mut map)) = converted.first_mut() {
            map.insert("content".into(), json!(system));
        }
    }

    let converted_tools = req.tools.clone().map(convert_tools_deepseek);

    let mut body = Map::new();
    body.insert("model".into(), json!(model));
    body.insert("messages".into(), Value::Array(converted));
    body.insert("stream".into(), json!(req.stream));
    if let Some(max_tokens) = req.max_tokens {
        body.insert("max_tokens".into(), json!(max_tokens));
    }
    if let Some(tools) = converted_tools {
        body.insert("tools".into(), Value::Array(tools));
        body.insert(
            "tool_choice".into(),
            req.tool_choice.clone().unwrap_or(json!("auto")),
        );
    }

    let thinking_param = req.extra.get("thinking").cloned();
    let reasoning_effort = req
        .extra
        .get("reasoning_effort")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let mut thinking_active = false;
    if DeepSeekBot::model_supports_thinking(model) {
        let thinking = thinking_param.unwrap_or(json!({"type": "enabled"}));
        let enabled = thinking.get("type").and_then(|t| t.as_str()) == Some("enabled");
        body.insert("thinking".into(), thinking);
        thinking_active = enabled;
        if thinking_active {
            body.insert(
                "reasoning_effort".into(),
                json!(reasoning_effort.unwrap_or_else(|| "high".to_string())),
            );
        }
    } else if DeepSeekBot::is_reasoner_model(model) {
        thinking_active = true;
    }

    if thinking_active {
        for k in [
            "temperature",
            "top_p",
            "presence_penalty",
            "frequency_penalty",
        ] {
            body.remove(k);
        }
    } else {
        if let Some(t) = req.temperature {
            body.insert("temperature".into(), json!(t));
        }
        if let Some(t) = req.top_p {
            body.insert("top_p".into(), json!(t));
        }
    }

    debug!(
        model = %model,
        tools = body.get("tools").and_then(|t| t.as_array()).map(|a| a.len()).unwrap_or(0),
        stream = req.stream,
        thinking = thinking_active,
        "deepseek call_with_tools"
    );

    body
}

fn convert_tools_deepseek(tools: Vec<Value>) -> Vec<Value> {
    let mut out = Vec::new();
    for tool in tools {
        if tool.get("type").and_then(|t| t.as_str()) == Some("function") {
            out.push(tool);
        } else {
            out.push(json!({
                "type": "function",
                "function": {
                    "name": tool.get("name").cloned().unwrap_or(Value::Null),
                    "description": tool.get("description").cloned().unwrap_or(Value::Null),
                    "parameters": tool.get("input_schema").cloned().unwrap_or_else(|| json!({})),
                }
            }));
        }
    }
    out
}
