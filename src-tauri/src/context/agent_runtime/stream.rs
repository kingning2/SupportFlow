//! Agent 消息流式执行与取消注册。

use std::sync::Arc;

use crate::agent::{get_cancel_registry, AgentEventCallback, CancelHandle, RunStreamOptions};
use crate::events::names::{AGENT_RUN_FINISHED, AGENT_STREAM_CHUNK};
use crate::events::payloads::{AgentRunFinished, AgentStreamChunk};
use tauri::{AppHandle, Emitter};

use super::AgentRuntime;

/// Run one agent message roundtrip and emit stream lifecycle events.
pub async fn run_agent_message(
    app: AppHandle,
    runtime: Arc<AgentRuntime>,
    request_id: String,
    message: String,
    cancel: CancelHandle,
) {
    if runtime.ensure_agent().await.is_err() {
        let _ = app.emit(
            AGENT_RUN_FINISHED,
            AgentRunFinished {
                request_id,
                error: Some("failed to initialize agent (check config.json / API key)".into()),
                content: None,
            },
        );
        return;
    }

    let request_id_cb = request_id.clone();
    let app_emit = app.clone();
    let on_event: AgentEventCallback = Arc::new(move |ev| {
        if let Some(chunk) = AgentRuntime::map_stream_event(&request_id_cb, &ev) {
            let _ = app_emit.emit(AGENT_STREAM_CHUNK, chunk);
        }
    });

    let run_result = async {
        let agent = runtime.ensure_agent().await?;
        agent
            .run_stream(
                &message,
                RunStreamOptions {
                    on_event: Some(on_event),
                    clear_history: false,
                    cancel: Some(cancel),
                    skill_filter: None,
                },
            )
            .await
            .map_err(|e| e.to_string())
    }
    .await;

    match run_result {
        Ok(content) => {
            let _ = app.emit(
                AGENT_STREAM_CHUNK,
                AgentStreamChunk {
                    request_id: request_id.clone(),
                    chunk_type: "done".into(),
                    content: Some(content.clone()),
                    tool: None,
                    arguments: None,
                    status: None,
                    result: None,
                    execution_time: None,
                },
            );
            let _ = app.emit(
                AGENT_RUN_FINISHED,
                AgentRunFinished {
                    request_id,
                    error: None,
                    content: Some(content),
                },
            );
        }
        Err(e) => {
            let msg = e.to_string();
            let _ = app.emit(
                AGENT_RUN_FINISHED,
                AgentRunFinished {
                    request_id,
                    error: Some(msg),
                    content: None,
                },
            );
        }
    }
}

pub fn register_cancel(request_id: &str, session_id: Option<&str>) -> CancelHandle {
    get_cancel_registry().register(request_id, session_id)
}

pub fn cancel_request(request_id: &str) {
    get_cancel_registry().cancel_request(request_id);
}
