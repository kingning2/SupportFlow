//! rig `PromptHook` — 将流式事件映射为既有 `AgentEvent` 回调。

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use rig_core::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig_core::completion::{CompletionModel, CompletionResponse};
use rig_core::message::Message;
use serde_json::{json, Value};

use crate::services::agent::protocol::AgentEvent;
use crate::services::agent::protocol::{AgentCancelledError, CancelHandle};

/// 将 rig 流式生命周期事件桥接到桌面端 `AgentEvent` 契约。
#[derive(Clone)]
pub struct RigStreamHook {
    pub on_event: Option<Arc<dyn Fn(AgentEvent) + Send + Sync>>,
    pub cancel: Option<CancelHandle>,
    pub files_to_send: Arc<std::sync::Mutex<Vec<Value>>>,
}

impl RigStreamHook {
    /// 创建流式事件 hook。
    pub fn new(
        on_event: Option<Arc<dyn Fn(AgentEvent) + Send + Sync>>,
        cancel: Option<CancelHandle>,
        files_to_send: Arc<std::sync::Mutex<Vec<Value>>>,
    ) -> Self {
        Self {
            on_event,
            cancel,
            files_to_send,
        }
    }

    fn emit(&self, event_type: &str, data: Value) {
        if let Some(cb) = &self.on_event {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            cb(AgentEvent {
                event_type: event_type.to_string(),
                timestamp: ts,
                data,
            });
        }
    }

    fn check_cancelled(&self) -> Result<(), AgentCancelledError> {
        if self.cancel.as_ref().is_some_and(|h| h.is_cancelled()) {
            return Err(AgentCancelledError);
        }
        Ok(())
    }
}

impl<M> PromptHook<M> for RigStreamHook
where
    M: CompletionModel,
{
    fn on_completion_call(
        &self,
        _prompt: &Message,
        _history: &[Message],
    ) -> impl std::future::Future<Output = HookAction> + rig_core::wasm_compat::WasmCompatSend {
        let this = self.clone();
        async move {
            if this.check_cancelled().is_err() {
                return HookAction::terminate("cancelled");
            }
            HookAction::cont()
        }
    }

    fn on_text_delta(
        &self,
        text_delta: &str,
        _aggregated_text: &str,
    ) -> impl std::future::Future<Output = HookAction> + Send {
        let this = self.clone();
        let delta = text_delta.to_string();
        async move {
            if this.check_cancelled().is_err() {
                return HookAction::terminate("cancelled");
            }
            this.emit("message_update", json!({ "delta": delta }));
            HookAction::cont()
        }
    }

    fn on_tool_call(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
    ) -> impl std::future::Future<Output = ToolCallHookAction> + rig_core::wasm_compat::WasmCompatSend
    {
        let this = self.clone();
        let tool_name = tool_name.to_string();
        let args: Value = serde_json::from_str(args).unwrap_or(json!({}));
        async move {
            if this.check_cancelled().is_err() {
                return ToolCallHookAction::terminate("cancelled");
            }
            this.emit(
                "tool_execution_start",
                json!({
                    "tool_name": tool_name,
                    "arguments": args,
                }),
            );
            ToolCallHookAction::cont()
        }
    }

    fn on_tool_result(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
        result: &str,
    ) -> impl std::future::Future<Output = HookAction> + rig_core::wasm_compat::WasmCompatSend {
        let this = self.clone();
        let tool_name = tool_name.to_string();
        let args: Value = serde_json::from_str(args).unwrap_or(json!({}));
        let result = result.to_string();
        async move {
            if tool_name == "send" {
                if let Ok(parsed) = serde_json::from_str::<Value>(&result) {
                    if parsed.get("path").is_some() {
                        if let Ok(mut files) = this.files_to_send.lock() {
                            files.push(parsed);
                        }
                    }
                }
            }
            this.emit(
                "tool_execution_end",
                json!({
                    "tool_name": tool_name,
                    "arguments": args,
                    "status": "success",
                    "result": result,
                    "execution_time": 0.0,
                }),
            );
            HookAction::cont()
        }
    }

    fn on_stream_completion_response_finish(
        &self,
        _prompt: &Message,
        _response: &M::StreamingResponse,
    ) -> impl std::future::Future<Output = HookAction> + Send {
        async { HookAction::cont() }
    }

    fn on_completion_response(
        &self,
        _prompt: &Message,
        _response: &CompletionResponse<M::Response>,
    ) -> impl std::future::Future<Output = HookAction> + rig_core::wasm_compat::WasmCompatSend {
        async { HookAction::cont() }
    }
}
