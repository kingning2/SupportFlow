//! Agent console IPC — wraps in-process `agent` crate.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use typeshare::typeshare;

use crate::context::agent_runtime::{self, AgentRuntime};
use crate::events::payloads::{AgentConsoleState, SkillItem};

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentSendMessageRequest {
    pub message: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentSendMessageResponse {
    pub request_id: String,
    pub session_id: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCancelRequest {
    pub request_id: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentUpdateProviderRequest {
    pub provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,
    #[serde(default)]
    pub api_base_set: bool,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentClearProviderRequest {
    pub provider_id: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentSetChatModelRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[tauri::command]
/// Load aggregated Agent console state for frontend bootstrap.
pub async fn agent_get_console_state(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentConsoleState, String> {
    runtime.console_state().await
}

#[tauri::command]
/// Submit a user message and start background streaming for the active session.
pub async fn agent_send_message(
    app: AppHandle,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentSendMessageRequest,
) -> Result<AgentSendMessageResponse, String> {
    let (request_id, session_id) = runtime
        .inner()
        .clone()
        .send_message(app, body.message)
        .await?;

    Ok(AgentSendMessageResponse {
        request_id,
        session_id,
    })
}

#[tauri::command]
/// Cancel a running agent request by request id.
pub fn agent_cancel(_app: AppHandle, body: AgentCancelRequest) -> Result<(), String> {
    agent_runtime::cancel_request(&body.request_id);
    Ok(())
}

#[tauri::command]
/// Clear in-memory conversation context for the current runtime session.
pub async fn agent_clear_context(runtime: State<'_, Arc<AgentRuntime>>) -> Result<(), String> {
    runtime.clear_context().await
}

#[tauri::command]
/// Create and switch to a new runtime session id.
pub async fn agent_new_session(runtime: State<'_, Arc<AgentRuntime>>) -> Result<String, String> {
    Ok(runtime.new_session().await)
}

#[tauri::command]
/// Update provider credentials and optional api base in bundled config.
pub async fn agent_update_provider(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentUpdateProviderRequest,
) -> Result<(), String> {
    runtime
        .update_provider(
            &body.provider_id,
            body.api_key.as_deref(),
            body.api_base.as_deref(),
            body.api_base_set,
        )
        .await
}

#[tauri::command]
/// Clear provider credentials in bundled config.
pub async fn agent_clear_provider(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentClearProviderRequest,
) -> Result<(), String> {
    runtime.clear_provider(&body.provider_id).await
}

#[tauri::command]
/// Set active chat provider/model pair in bundled config.
pub async fn agent_set_chat_model(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentSetChatModelRequest,
) -> Result<(), String> {
    runtime
        .set_active_chat(body.provider_id.as_deref(), body.model.as_deref())
        .await
}

#[tauri::command]
/// Refresh skill registry and return latest skill list.
pub async fn agent_refresh_skills(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<SkillItem>, String> {
    runtime.refresh_skills().await
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentMemoryItem {
    pub filename: String,
    pub item_type: String,
    pub size: i32,
    pub updated_at: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentMemoryReadRequest {
    pub filename: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentMemoryReadResult {
    pub filename: String,
    pub content: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeFile {
    pub path: String,
    pub title: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeReadRequest {
    pub path: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeReadResult {
    pub path: String,
    pub content: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeGraphNode {
    pub id: String,
    pub label: String,
    pub category: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeGraphLink {
    pub source: String,
    pub target: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeGraph {
    pub nodes: Vec<AgentKnowledgeGraphNode>,
    pub links: Vec<AgentKnowledgeGraphLink>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentChannelSummary {
    pub name: String,
    pub active: bool,
    pub label: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentChannelField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentChannelDetail {
    pub name: String,
    pub label_key: String,
    pub active: bool,
    pub fields: Vec<AgentChannelField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint_key: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentChannelActionRequest {
    pub action: String,
    pub channel: String,
    #[serde(default)]
    pub config: std::collections::HashMap<String, serde_json::Value>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentChannelActionResponse {
    pub channel_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChannelConsoleApiRequest {
    pub path: String,
    pub method: String,
    #[serde(default)]
    pub body: serde_json::Value,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskSummary {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_run_at: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentLogsStatus {
    pub enabled: bool,
    pub source: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentReadLogsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentReadLogsResult {
    pub source: String,
    pub content: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentLogStreamState {
    pub started: bool,
}

#[tauri::command]
/// List persisted sessions from workspace index.
pub async fn agent_list_sessions(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentSessionSummary>, String> {
    let rows = runtime.list_sessions().await?;
    Ok(rows
        .into_iter()
        .map(|s| AgentSessionSummary {
            id: s.id,
            title: s.title,
            updated_at: s.updated_at,
        })
        .collect())
}

#[tauri::command]
/// List memory files from runtime workspace.
pub async fn agent_list_memory(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentMemoryItem>, String> {
    let rows = runtime.list_memory_items().await?;
    Ok(rows
        .into_iter()
        .map(|m| AgentMemoryItem {
            filename: m.filename,
            item_type: m.item_type,
            size: m.size,
            updated_at: m.updated_at,
        })
        .collect())
}

#[tauri::command]
/// Read memory file content by filename.
pub async fn agent_read_memory(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentMemoryReadRequest,
) -> Result<AgentMemoryReadResult, String> {
    let content = runtime.read_memory_item(&body.filename).await?;
    Ok(AgentMemoryReadResult {
        filename: body.filename,
        content,
    })
}

#[tauri::command]
/// List knowledge documents under workspace/knowledge.
pub async fn agent_list_knowledge(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentKnowledgeFile>, String> {
    let rows = runtime.list_knowledge_files().await?;
    Ok(rows
        .into_iter()
        .map(|f| AgentKnowledgeFile {
            path: f.path,
            title: f.title,
        })
        .collect())
}

#[tauri::command]
/// Read one knowledge document by relative path.
pub async fn agent_read_knowledge(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentKnowledgeReadRequest,
) -> Result<AgentKnowledgeReadResult, String> {
    let content = runtime.read_knowledge_file(&body.path).await?;
    Ok(AgentKnowledgeReadResult {
        path: body.path,
        content,
    })
}

#[tauri::command]
/// Return knowledge graph nodes and links from markdown cross-references.
pub async fn agent_get_knowledge_graph(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentKnowledgeGraph, String> {
    let graph = runtime.knowledge_graph().await?;
    Ok(AgentKnowledgeGraph {
        nodes: graph
            .nodes
            .into_iter()
            .map(|n| AgentKnowledgeGraphNode {
                id: n.id,
                label: n.label,
                category: n.category,
            })
            .collect(),
        links: graph
            .links
            .into_iter()
            .map(|l| AgentKnowledgeGraphLink {
                source: l.source,
                target: l.target,
            })
            .collect(),
    })
}

#[tauri::command]
/// List active channel summaries (legacy/simple list).
pub async fn agent_list_channels(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentChannelSummary>, String> {
    let rows = runtime.list_channels().await?;
    Ok(rows
        .into_iter()
        .map(|c| AgentChannelSummary {
            name: c.name,
            active: c.active,
            label: c.label,
        })
        .collect())
}

#[tauri::command]
/// Channel catalog proxied to CowAgent Python `GET /api/channels`.
pub async fn agent_get_channel_catalog(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<serde_json::Value, String> {
    runtime.cow_python_channels_get().await
}

#[tauri::command]
/// Channel connect/disconnect/save proxied to CowAgent Python `POST /api/channels`.
pub async fn agent_channel_action(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentChannelActionRequest,
) -> Result<serde_json::Value, String> {
    let payload = serde_json::json!({
        "action": body.action,
        "channel": body.channel,
        "config": body.config,
    });
    runtime.cow_python_channels_post(payload).await
}

#[tauri::command]
/// Channel console APIs (QR login, Feishu register) proxied to Python sidecar.
pub async fn agent_channel_console_api(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentChannelConsoleApiRequest,
) -> Result<serde_json::Value, String> {
    runtime
        .cow_channel_console_api(&body.path, &body.method, body.body)
        .await
}

#[tauri::command]
/// List scheduled task summaries from runtime workspace.
pub async fn agent_list_tasks(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentTaskSummary>, String> {
    let rows = runtime.list_task_items().await?;
    Ok(rows
        .into_iter()
        .map(|t| AgentTaskSummary {
            id: t.id,
            name: t.name,
            enabled: t.enabled,
            next_run_at: t.next_run_at,
        })
        .collect())
}

#[tauri::command]
/// Return current log source path and availability for console log view.
pub async fn agent_get_logs_status(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentLogsStatus, String> {
    let (enabled, source) = runtime.logs_status().await?;
    Ok(AgentLogsStatus { enabled, source })
}

#[tauri::command]
/// Start background log tailing and push deltas through AGENT_LOG_STREAM event.
pub async fn agent_start_log_stream(
    app: AppHandle,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentLogStreamState, String> {
    let started = runtime.inner().clone().start_log_stream(app).await?;
    Ok(AgentLogStreamState { started })
}

#[tauri::command]
/// Stop background log tailing loop.
pub async fn agent_stop_log_stream(
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentLogStreamState, String> {
    runtime.stop_log_stream().await;
    Ok(AgentLogStreamState { started: false })
}

#[tauri::command]
/// Read latest log lines with optional line limit.
pub async fn agent_read_logs(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: Option<AgentReadLogsRequest>,
) -> Result<AgentReadLogsResult, String> {
    let (source, content) = runtime.read_logs(body.and_then(|b| b.limit)).await?;

    Ok(AgentReadLogsResult { source, content })
}
