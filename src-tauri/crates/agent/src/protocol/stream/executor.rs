//! `agent/protocol/agent_stream.py` — `AgentStreamExecutor` (LLM streaming core).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use serde_json::{json, Map, Value};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::protocol::stream::helpers::{
    chunk_is_top_level_error, extract_stream_error, filter_think_tags, is_context_overflow_error,
    is_message_format_error, is_rate_limit_error, is_retryable_llm_error, parse_tool_args,
    truncate_reasoning_for_storage,
};
use crate::protocol::stream::llm_bridge::{LlmBridgeConfig, LlmModel};
use crate::protocol::stream::turns::{aggressive_trim_for_overflow, identify_complete_turns};
use crate::protocol::{sanitize_claude_messages, AgentCancelledError, CancelHandle, LlmRequest};
use crate::tools::{AgentTool, McpToolRegistry, ToolRunResult};

const CANCEL_PROBE_EVERY: u32 = 8;

pub type AgentEventCallback = Arc<dyn Fn(AgentEvent) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct AgentEvent {
    pub event_type: String,
    pub timestamp: f64,
    pub data: Value,
}

#[derive(Debug, Clone)]
pub struct AgentToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    pub parse_error: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CallLlmError {
    #[error("agent cancelled")]
    Cancelled(#[from] AgentCancelledError),
    #[error("{0}")]
    Failed(String),
}

/// Optional hooks from the outer `Agent` (memory flush, DB clear).
pub trait AgentStreamHost: Send + Sync {
    fn flush_memory_overflow(&self, _messages: &[Value]) {}
    fn clear_session_db(&self) {}
    fn context_window_tokens(&self) -> u32 {
        128_000
    }
    fn max_context_tokens(&self) -> Option<u32> {
        None
    }
    fn memory_flush_on_trim(
        &self,
        _discarded_messages: &[Value],
        _reason: &str,
        _discarded_turn_count: usize,
    ) {
    }
}

/// Schema-only tool placeholder (LLM sees definition; execute returns error until real tool wired).
pub struct SchemaStubTool {
    pub schema: AgentToolSchema,
}

#[async_trait::async_trait]
impl AgentTool for SchemaStubTool {
    fn name(&self) -> &str {
        &self.schema.name
    }

    fn description(&self) -> &str {
        &self.schema.description
    }

    fn input_schema(&self) -> Value {
        self.schema.input_schema.clone()
    }

    async fn execute(&self, _params: Value) -> ToolRunResult {
        ToolRunResult::error(format!(
            "Tool '{}' is registered for LLM only (not implemented yet)",
            self.schema.name
        ))
    }
}

/// Build runtime tools from JSON schemas (tests / gradual migration).
pub fn tools_from_schemas(schemas: Vec<AgentToolSchema>) -> Vec<Arc<dyn AgentTool>> {
    schemas
        .into_iter()
        .map(|schema| Arc::new(SchemaStubTool { schema }) as Arc<dyn AgentTool>)
        .collect()
}

struct ToolCallBuffer {
    id: String,
    name: String,
    arguments: String,
}

/// Multi-turn stream executor — step 3 ports `_call_llm_stream` and supporting methods.
pub struct AgentStreamExecutor {
    pub model: Arc<dyn LlmModel>,
    pub bridge: LlmBridgeConfig,
    pub system_prompt: String,
    pub tools: HashMap<String, Arc<dyn AgentTool>>,
    pub messages: Vec<Value>,
    pub max_turns: u32,
    pub max_context_turns: u32,
    pub on_event: Option<AgentEventCallback>,
    pub cancel: Option<CancelHandle>,
    pub host: Option<Arc<dyn AgentStreamHost>>,
    pub mcp_registry: Option<Arc<McpToolRegistry>>,
    pub tool_failure_history: Vec<(String, String, bool)>,
    pub files_to_send: Vec<Value>,
}

impl AgentStreamExecutor {
    pub fn new(
        model: Arc<dyn LlmModel>,
        bridge: LlmBridgeConfig,
        system_prompt: impl Into<String>,
        tools: Vec<Arc<dyn AgentTool>>,
        max_turns: u32,
        on_event: Option<AgentEventCallback>,
        messages: Option<Vec<Value>>,
        max_context_turns: u32,
        cancel: Option<CancelHandle>,
        host: Option<Arc<dyn AgentStreamHost>>,
    ) -> Self {
        let tools_map = tools
            .into_iter()
            .map(|t| (t.name().to_string(), t))
            .collect();
        Self {
            model,
            bridge,
            system_prompt: system_prompt.into(),
            tools: tools_map,
            messages: messages.unwrap_or_default(),
            max_turns,
            max_context_turns,
            on_event,
            cancel,
            host,
            mcp_registry: None,
            tool_failure_history: Vec::new(),
            files_to_send: Vec::new(),
        }
    }

    /// Pull MCP tools that finished loading since the last turn (`ToolManager.sync_mcp_into_agent`).
    pub fn sync_mcp_tools(&mut self) {
        if let Some(registry) = &self.mcp_registry {
            registry.sync_into(&mut self.tools);
        }
    }

    pub fn check_cancelled(&self) -> Result<(), AgentCancelledError> {
        if self.cancel.as_ref().is_some_and(|h| h.is_cancelled()) {
            return Err(AgentCancelledError);
        }
        Ok(())
    }

    pub fn handle_cancelled(&mut self, _partial_response: &str) {
        if let Some(last) = self.messages.last() {
            if last.get("role") == Some(&Value::String("assistant".into())) {
                if let Some(blocks) = last.get("content").and_then(|c| c.as_array()) {
                    let pending: Vec<String> = blocks
                        .iter()
                        .filter_map(|b| {
                            if b.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                                b.get("id").and_then(|id| id.as_str()).map(str::to_string)
                            } else {
                                None
                            }
                        })
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !pending.is_empty() {
                        let tool_result_blocks: Vec<Value> = pending
                            .iter()
                            .map(|tid| {
                                json!({
                                    "type": "tool_result",
                                    "tool_use_id": tid,
                                    "content": "Cancelled by user before this tool finished.",
                                    "is_error": true,
                                })
                            })
                            .collect();
                        self.messages.push(json!({
                            "role": "user",
                            "content": tool_result_blocks,
                        }));
                        info!(
                            count = pending.len(),
                            "Injected cancellation tool_result blocks"
                        );
                    }
                }
            }
        }
        self.messages.push(json!({
            "role": "assistant",
            "content": [{"type": "text", "text": "_(Cancelled by user)_"}],
        }));
    }

    pub(crate) fn emit_event(&self, event_type: &str, data: Value) {
        if let Some(cb) = &self.on_event {
            let event = AgentEvent {
                event_type: event_type.to_string(),
                timestamp: now_secs(),
                data,
            };
            cb(event);
        }
    }

    pub(crate) fn is_thinking_enabled(&self) -> bool {
        self.bridge.enable_thinking
    }

    fn should_render_thinking_inline(&self) -> bool {
        self.bridge.enable_thinking && self.model.channel_type() == "web"
    }

    pub fn validate_and_fix_messages(&mut self) {
        sanitize_claude_messages(&mut self.messages);
    }

    pub fn prepare_messages(&self) -> Vec<Value> {
        self.messages.clone()
    }

    fn clear_session_db(&self) {
        if let Some(host) = &self.host {
            host.clear_session_db();
        }
    }

    fn build_tools_schema(&self) -> Option<Vec<Value>> {
        if self.tools.is_empty() {
            return None;
        }
        Some(
            self.tools
                .values()
                .map(|t| {
                    json!({
                        "name": t.name(),
                        "description": t.description(),
                        "input_schema": t.input_schema(),
                    })
                })
                .collect(),
        )
    }

    /// `AgentStreamExecutor._call_llm_stream`
    pub async fn call_llm_stream(
        &mut self,
        retry_on_empty: bool,
        retry_count: u32,
        max_retries: u32,
        overflow_retry: bool,
    ) -> Result<(String, Vec<ParsedToolCall>), CallLlmError> {
        self.validate_and_fix_messages();
        self.sync_mcp_tools();

        let messages = self.prepare_messages();
        let turns = identify_complete_turns(&messages);
        info!(
            msg_count = messages.len(),
            turn_count = turns.len(),
            "Sending messages to LLM"
        );

        let request = LlmRequest {
            messages,
            temperature: 0.0,
            stream: true,
            tools: self.build_tools_schema(),
            system: Some(self.system_prompt.clone()),
            ..Default::default()
        };

        self.emit_event("message_start", json!({ "role": "assistant" }));

        let mut full_content = String::new();
        let mut full_reasoning = String::new();
        let mut tool_calls_buffer: HashMap<u32, ToolCallBuffer> = HashMap::new();
        let mut gemini_raw_parts: Option<Value> = None;
        let mut stop_reason: Option<String> = None;

        let stream_result = async {
            let mut stream = self
                .model
                .call_stream(&request)
                .await
                .map_err(|e| CallLlmError::Failed(e.to_string()))?;

            let mut cancel_probe = 0u32;

            while let Some(chunk) = stream.next().await {
                cancel_probe += 1;
                if cancel_probe >= CANCEL_PROBE_EVERY {
                    cancel_probe = 0;
                    if self
                        .cancel
                        .as_ref()
                        .is_some_and(|h| h.is_cancelled())
                    {
                        info!("cancel detected mid-stream, aborting LLM call");
                        if !full_content.is_empty() {
                            self.messages.push(json!({
                                "role": "assistant",
                                "content": [{"type": "text", "text": full_content}],
                            }));
                        }
                        self.emit_event(
                            "message_end",
                            json!({
                                "content": full_content,
                                "tool_calls": [],
                                "cancelled": true,
                            }),
                        );
                        return Err(CallLlmError::Cancelled(AgentCancelledError));
                    }
                }

                if chunk_is_top_level_error(&chunk) {
                    let (error_msg, error_code, error_type, status_code) =
                        extract_stream_error(&chunk);
                    error!(
                        message = %error_msg,
                        status_code,
                        error_code = %error_code,
                        error_type = %error_type,
                        ?chunk,
                        "Stream API Error"
                    );
                    if is_context_overflow_error(&error_msg) {
                        return Err(CallLlmError::Failed(format!(
                            "[CONTEXT_OVERFLOW] {error_msg} (Status: {status_code})"
                        )));
                    }
                    return Err(CallLlmError::Failed(format!(
                        "{error_msg} (Status: {status_code}, Code: {error_code}, Type: {error_type})"
                    )));
                }

                let choices = match chunk.get("choices").and_then(|c| c.as_array()) {
                    Some(c) if !c.is_empty() => c,
                    _ => continue,
                };
                let choice = &choices[0];
                let delta = choice.get("delta").cloned().unwrap_or(Value::Null);

                if let Some(fr) = choice.get("finish_reason").and_then(|v| v.as_str()) {
                    stop_reason = Some(fr.to_string());
                }

                if let Some(r) = delta.get("reasoning_content").and_then(|v| v.as_str()) {
                    if !r.is_empty() {
                        full_reasoning.push_str(r);
                        if self.is_thinking_enabled() {
                            self.emit_event(
                                "reasoning_update",
                                json!({ "delta": r }),
                            );
                        }
                    }
                }

                if let Some(c) = delta.get("content").and_then(|v| v.as_str()) {
                    if !c.is_empty() {
                        let filtered =
                            filter_think_tags(c, self.should_render_thinking_inline());
                        full_content.push_str(&filtered);
                        if !filtered.is_empty() {
                            self.emit_event(
                                "message_update",
                                json!({ "delta": filtered }),
                            );
                        }
                    }
                }

                if let Some(tcs) = delta.get("tool_calls").and_then(|v| v.as_array()) {
                    for tc_delta in tcs {
                        let index = tc_delta
                            .get("index")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;
                        let entry = tool_calls_buffer.entry(index).or_insert(ToolCallBuffer {
                            id: String::new(),
                            name: String::new(),
                            arguments: String::new(),
                        });
                        if let Some(id) = tc_delta.get("id").and_then(|v| v.as_str()) {
                            entry.id = id.to_string();
                        }
                        if let Some(func) = tc_delta.get("function") {
                            if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                                entry.name = name.to_string();
                            }
                            if let Some(args) = func.get("arguments").and_then(|v| v.as_str()) {
                                entry.arguments.push_str(args);
                            }
                        }
                    }
                }

                if delta.get("_gemini_raw_parts").is_some() {
                    gemini_raw_parts = delta.get("_gemini_raw_parts").cloned();
                } else if choice.get("_gemini_raw_parts").is_some() {
                    gemini_raw_parts = choice.get("_gemini_raw_parts").cloned();
                }
            }
            Ok(())
        }
        .await;

        match stream_result {
            Err(CallLlmError::Cancelled(e)) => return Err(CallLlmError::Cancelled(e)),
            Err(CallLlmError::Failed(e)) => {
                let error_str = e.clone();
                let is_overflow = is_context_overflow_error(&error_str);
                let is_format = is_message_format_error(&error_str);

                if is_overflow || is_format {
                    let error_type = if is_overflow {
                        "context overflow"
                    } else {
                        "message format error"
                    };
                    error!(error_type, error = %error_str, "LLM context/format error");

                    if is_overflow {
                        if let Some(host) = &self.host {
                            host.flush_memory_overflow(&self.messages);
                        }
                    }

                    if is_overflow && !overflow_retry {
                        if aggressive_trim_for_overflow(&mut self.messages) {
                            warn!("Aggressively trimmed context, retrying...");
                            return Box::pin(self.call_llm_stream(
                                retry_on_empty,
                                retry_count,
                                max_retries,
                                true,
                            ))
                            .await;
                        }
                    }

                    warn!("Clearing conversation history to recover");
                    self.messages.clear();
                    self.clear_session_db();
                    let msg = if is_overflow {
                        "抱歉，对话历史过长导致上下文溢出。我已清空历史记录，请重新描述你的需求。"
                    } else {
                        "抱歉，之前的对话出现了问题。我已清空历史记录，请重新发送你的消息。"
                    };
                    return Err(CallLlmError::Failed(msg.to_string()));
                }

                if is_retryable_llm_error(&error_str) && retry_count < max_retries {
                    let wait = if is_rate_limit_error(&error_str) {
                        30 + retry_count * 15
                    } else {
                        (retry_count + 1) * 2
                    };
                    warn!(
                        attempt = retry_count + 1,
                        max_retries,
                        wait_secs = wait,
                        error = %error_str,
                        "LLM API error, retrying"
                    );
                    sleep(Duration::from_secs(wait as u64)).await;
                    return Box::pin(self.call_llm_stream(
                        retry_on_empty,
                        retry_count + 1,
                        max_retries,
                        overflow_retry,
                    ))
                    .await;
                }

                if retry_count >= max_retries {
                    error!(error = %error_str, "LLM API error after max retries");
                } else {
                    error!(error = %error_str, "LLM call error (non-retryable)");
                }
                return Err(CallLlmError::Failed(error_str));
            }
            Ok(()) => {}
        }

        let stop_reason_ref = stop_reason.as_deref();
        let mut tool_calls = Vec::new();
        let mut indices: Vec<u32> = tool_calls_buffer.keys().copied().collect();
        indices.sort_unstable();
        for idx in indices {
            let tc = tool_calls_buffer.remove(&idx).expect("buffer entry");
            let tool_id = if tc.id.is_empty() {
                format!("call_{:x}", Uuid::new_v4().as_u128())
                    .chars()
                    .take(26)
                    .collect::<String>()
            } else {
                tc.id
            };
            let (arguments, parse_err) = parse_tool_args(&tc.arguments, stop_reason_ref);
            if let Some(err) = parse_err {
                error!(
                    tool = %tc.name,
                    arg_len = tc.arguments.len(),
                    error = %err,
                    "Tool args parse failed"
                );
                tool_calls.push(ParsedToolCall {
                    id: tool_id,
                    name: tc.name,
                    arguments: Value::Object(Map::new()),
                    parse_error: Some(err),
                });
                continue;
            }
            tool_calls.push(ParsedToolCall {
                id: tool_id,
                name: tc.name,
                arguments,
                parse_error: None,
            });
        }

        if retry_on_empty && full_content.is_empty() && tool_calls.is_empty() {
            warn!(?stop_reason, "LLM returned empty response, retrying once");
            self.emit_event(
                "message_end",
                json!({
                    "content": "",
                    "tool_calls": [],
                    "empty_retry": true,
                    "stop_reason": stop_reason,
                }),
            );
            return Box::pin(self.call_llm_stream(false, retry_count, max_retries, overflow_retry))
                .await;
        }

        full_content = filter_think_tags(&full_content, self.should_render_thinking_inline());

        let mut content_arr: Vec<Value> = Vec::new();

        if !full_reasoning.is_empty() {
            let stored = truncate_reasoning_for_storage(&full_reasoning);
            if stored.len() < full_reasoning.len() {
                info!(
                    from = full_reasoning.len(),
                    to = stored.len(),
                    "reasoning truncated for storage"
                );
            }
            content_arr.push(json!({
                "type": "thinking",
                "thinking": stored,
            }));
        }
        if !full_content.is_empty() {
            content_arr.push(json!({
                "type": "text",
                "text": full_content,
            }));
        }
        for tc in &tool_calls {
            content_arr.push(json!({
                "type": "tool_use",
                "id": tc.id,
                "name": tc.name,
                "input": tc.arguments,
            }));
        }

        if !content_arr.is_empty() {
            let mut assistant_msg = json!({
                "role": "assistant",
                "content": content_arr,
            });
            if let Some(parts) = gemini_raw_parts {
                assistant_msg
                    .as_object_mut()
                    .expect("assistant")
                    .insert("_gemini_raw_parts".into(), parts);
            }
            self.messages.push(assistant_msg);
        }

        let tool_calls_json: Vec<Value> = tool_calls
            .iter()
            .map(|tc| {
                json!({
                    "id": tc.id,
                    "name": tc.name,
                    "arguments": tc.arguments,
                })
            })
            .collect();

        self.emit_event(
            "message_end",
            json!({
                "content": full_content,
                "tool_calls": tool_calls_json,
            }),
        );

        Ok((full_content, tool_calls))
    }
}

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
