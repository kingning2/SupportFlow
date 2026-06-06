//! 渠道 sidecar 入站 RPC 分发（Python → Rust）。

use std::sync::{Arc, Weak};

use async_trait::async_trait;
use process_runtime::InboundRpcHandler;
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::context::agent_runtime::AgentRuntime;

pub struct ChannelInboundHandler {
    runtime: Mutex<Option<Weak<AgentRuntime>>>,
}

impl ChannelInboundHandler {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            runtime: Mutex::new(None),
        })
    }

    pub async fn register_runtime(&self, runtime: Weak<AgentRuntime>) {
        *self.runtime.lock().await = Some(runtime);
    }

    async fn runtime_handle(&self) -> Result<Arc<AgentRuntime>, String> {
        self.runtime
            .lock()
            .await
            .as_ref()
            .and_then(|runtime| runtime.upgrade())
            .ok_or_else(|| "AgentRuntime not registered".to_string())
    }
}

#[async_trait]
impl InboundRpcHandler for ChannelInboundHandler {
    async fn handle(&self, req: &Value) -> Value {
        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(json!({}));

        match method {
            "agent.reply" => match self.runtime_handle().await {
                Ok(rt) => match rt.channel_reply(&params).await {
                    Ok(reply) => json!({
                        "id": id,
                        "result": {
                            "status": "success",
                            "content": reply.get("content").cloned().unwrap_or(json!("")),
                            "reply_type": reply.get("reply_type").cloned(),
                            "text_content": reply.get("text_content").cloned(),
                            "file_name": reply.get("file_name").cloned(),
                        }
                    }),
                    Err(e) => json!({ "id": id, "error": e }),
                },
                Err(e) => json!({ "id": id, "error": e }),
            },
            "channel.process" => match self.runtime_handle().await {
                Ok(rt) => match rt.channel_process(&params).await {
                    Ok(payload) => json!({ "id": id, "result": payload }),
                    Err(e) => json!({ "id": id, "error": e }),
                },
                Err(e) => json!({ "id": id, "error": e }),
            },
            "channel.decorate_text" => match self.runtime_handle().await {
                Ok(rt) => match rt.channel_decorate_text(&params).await {
                    Ok(text) => json!({ "id": id, "result": { "text": text } }),
                    Err(e) => json!({ "id": id, "error": e }),
                },
                Err(e) => json!({ "id": id, "error": e }),
            },
            "channel.extract_media" => match self.runtime_handle().await {
                Ok(rt) => match rt.channel_extract_media(&params).await {
                    Ok(items) => json!({ "id": id, "result": { "items": items } }),
                    Err(e) => json!({ "id": id, "error": e }),
                },
                Err(e) => json!({ "id": id, "error": e }),
            },
            "channel.notify" => {
                if let Ok(rt) = self.runtime_handle().await {
                    rt.handle_channel_notification(&params);
                }
                json!({
                    "id": id,
                    "result": { "status": "success" }
                })
            }
            "channel.message" => json!({
                "id": id,
                "result": { "status": "success" }
            }),
            "wework.contacts_synced" => match self.runtime_handle().await {
                Ok(rt) => {
                    let wework_user_id = params
                        .get("wework_user_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if wework_user_id.is_empty() {
                        json!({ "id": id, "error": "wework_user_id required" })
                    } else {
                        match rt.wework_contacts_synced(wework_user_id) {
                            Ok(value) => json!({ "id": id, "result": { "value": value } }),
                            Err(e) => json!({ "id": id, "error": e }),
                        }
                    }
                }
                Err(e) => json!({ "id": id, "error": e }),
            },
            "wework.mark_contacts_synced" => match self.runtime_handle().await {
                Ok(rt) => {
                    let wework_user_id = params
                        .get("wework_user_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let synced_at = params
                        .get("synced_at")
                        .and_then(|v| v.as_i64())
                        .unwrap_or_default();
                    if wework_user_id.is_empty() || synced_at <= 0 {
                        json!({ "id": id, "error": "wework_user_id and synced_at required" })
                    } else {
                        match rt.wework_mark_contacts_synced(wework_user_id, synced_at) {
                            Ok(()) => json!({ "id": id, "result": { "status": "success" } }),
                            Err(e) => json!({ "id": id, "error": e }),
                        }
                    }
                }
                Err(e) => json!({ "id": id, "error": e }),
            },
            _ => json!({ "id": id, "error": format!("unknown method: {method}") }),
        }
    }
}
