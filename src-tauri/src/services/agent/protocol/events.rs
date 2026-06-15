//! Agent 流式事件类型（前端 IPC 契约）。

use std::sync::Arc;

use serde_json::Value;

/// 单次 agent 流式生命周期事件。
#[derive(Debug, Clone)]
pub struct AgentEvent {
    pub event_type: String,
    pub timestamp: f64,
    pub data: Value,
}

/// 流式事件回调。
pub type AgentEventCallback = Arc<dyn Fn(AgentEvent) + Send + Sync>;
