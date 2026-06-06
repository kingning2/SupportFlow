//! Agent console IPC — wraps in-process `agent` crate.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use typeshare::typeshare;

use crate::context::agent_runtime::{self, AgentRuntime};
use crate::context::license_store::LicenseStore;
use crate::events::payloads::{AgentConsoleState, SkillDetail, SkillItem};
use crate::utils::skills_installer::InstallSkillResult;

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

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentGetSkillDetailRequest {
    pub name: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentInstallSkillRequest {
    pub source: String,
}

#[tauri::command]
/// Load aggregated Agent console state for frontend bootstrap.
pub async fn agent_get_console_state(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentConsoleState, String> {
    license.require_valid()?;
    runtime.console_state().await
}

#[tauri::command]
/// Submit a user message and start background streaming for the active session.
pub async fn agent_send_message(
    app: AppHandle,
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentSendMessageRequest,
) -> Result<AgentSendMessageResponse, String> {
    license.require_valid()?;
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
pub fn agent_cancel(
    _app: AppHandle,
    license: tauri::State<'_, LicenseStore>,
    body: AgentCancelRequest,
) -> Result<(), String> {
    license.require_valid()?;
    agent_runtime::cancel_request(&body.request_id);
    Ok(())
}

#[tauri::command]
/// Clear in-memory conversation context for the current runtime session.
pub async fn agent_clear_context(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<(), String> {
    license.require_valid()?;
    runtime.clear_context().await
}

#[tauri::command]
/// Create and switch to a new runtime session id.
pub async fn agent_new_session(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<String, String> {
    license.require_valid()?;
    Ok(runtime.new_session().await)
}

#[tauri::command]
/// Update provider credentials and optional api base in bundled config.
pub async fn agent_update_provider(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentUpdateProviderRequest,
) -> Result<(), String> {
    license.require_valid()?;
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
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentClearProviderRequest,
) -> Result<(), String> {
    license.require_valid()?;
    runtime.clear_provider(&body.provider_id).await
}

#[tauri::command]
/// Set active chat provider/model pair in bundled config.
pub async fn agent_set_chat_model(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentSetChatModelRequest,
) -> Result<(), String> {
    license.require_valid()?;
    runtime
        .set_active_chat(body.provider_id.as_deref(), body.model.as_deref())
        .await
}

#[tauri::command]
/// Refresh skill registry and return latest skill list.
pub async fn agent_refresh_skills(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<SkillItem>, String> {
    license.require_valid()?;
    runtime.refresh_skills().await
}

#[tauri::command]
/// Load a single skill detail by skill name.
pub async fn agent_get_skill_detail(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentGetSkillDetailRequest,
) -> Result<SkillDetail, String> {
    license.require_valid()?;
    runtime.skill_detail(&body.name).await
}

#[tauri::command]
/// Install an external skill source and return the installed names.
pub async fn agent_install_skill(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentInstallSkillRequest,
) -> Result<InstallSkillResult, String> {
    license.require_valid()?;
    runtime.install_skill(&body.source).await
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
pub struct AgentKnowledgeUploadFile {
    pub filename: String,
    pub data: Vec<u8>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeUploadRequest {
    pub files: Vec<AgentKnowledgeUploadFile>,
    #[serde(default)]
    pub category: Option<String>,
}

/// Request for the picker command (category only; dialog is shown on Rust side).
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgePickUploadRequest {
    #[serde(default)]
    pub category: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeIngestItem {
    pub path: String,
    pub title: String,
    pub category: String,
    pub slug: String,
    pub original_name: String,
    pub truncated: bool,
    pub char_count: u32,
    pub archive: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeIngestError {
    pub file: String,
    pub message: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentKnowledgeUploadResult {
    pub results: Vec<AgentKnowledgeIngestItem>,
    pub errors: Vec<AgentKnowledgeIngestError>,
    pub count: u32,
    pub memory_synced: bool,
}

impl From<crate::agent::IngestBatchResult> for AgentKnowledgeUploadResult {
    fn from(batch: crate::agent::IngestBatchResult) -> Self {
        Self {
            results: batch
                .results
                .into_iter()
                .map(|r| AgentKnowledgeIngestItem {
                    path: r.path,
                    title: r.title,
                    category: r.category,
                    slug: r.slug,
                    original_name: r.original_name,
                    truncated: r.truncated,
                    char_count: r.char_count as u32,
                    archive: r.archive,
                })
                .collect(),
            errors: batch
                .errors
                .into_iter()
                .map(|e| AgentKnowledgeIngestError {
                    file: e.file,
                    message: e.message,
                })
                .collect(),
            count: batch.count as u32,
            memory_synced: batch.memory_synced,
        }
    }
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentSessionSummary>, String> {
    license.require_valid()?;
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
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentMemoryItem>, String> {
    license.require_valid()?;
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
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentMemoryReadRequest,
) -> Result<AgentMemoryReadResult, String> {
    license.require_valid()?;
    let content = runtime.read_memory_item(&body.filename).await?;
    Ok(AgentMemoryReadResult {
        filename: body.filename,
        content,
    })
}

#[tauri::command]
/// List knowledge documents under workspace/knowledge.
pub async fn agent_list_knowledge(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentKnowledgeFile>, String> {
    license.require_valid()?;
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
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentKnowledgeReadRequest,
) -> Result<AgentKnowledgeReadResult, String> {
    license.require_valid()?;
    let content = runtime.read_knowledge_file(&body.path).await?;
    Ok(AgentKnowledgeReadResult {
        path: body.path,
        content,
    })
}

#[tauri::command]
/// Remove one knowledge document by relative path.
pub async fn agent_remove_knowledge_file(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    path: String,
) -> Result<(), String> {
    license.require_valid()?;
    runtime.remove_knowledge_file(&path).await?;
    Ok(())
}

#[tauri::command]
/// Return knowledge graph nodes and links from markdown cross-references.
pub async fn agent_get_knowledge_graph(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentKnowledgeGraph, String> {
    license.require_valid()?;
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
/// Ingest uploaded files into knowledge/ (parse → Markdown → index/log → memory sync).
pub async fn agent_upload_knowledge(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentKnowledgeUploadRequest,
) -> Result<AgentKnowledgeUploadResult, String> {
    license.require_valid()?;
    let files: Vec<(String, Vec<u8>)> = body
        .files
        .into_iter()
        .map(|f| (f.filename, f.data))
        .collect();
    Ok(runtime
        .upload_knowledge_files(files, body.category.as_deref())
        .await?
        .into())
}

/// Pick supported knowledge files via the native OS dialog, then ingest them.
#[tauri::command]
pub async fn agent_pick_and_upload_knowledge(
    app: AppHandle,
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: Option<AgentKnowledgePickUploadRequest>,
) -> Result<AgentKnowledgeUploadResult, String> {
    license.require_valid()?;
    let category = body.and_then(|b| b.category);
    Ok(runtime
        .pick_and_upload_knowledge(&app, category.as_deref())
        .await?
        .into())
}

#[tauri::command]
/// List active channel summaries (legacy/simple list).
pub async fn agent_list_channels(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentChannelSummary>, String> {
    license.require_valid()?;
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
/// Channel catalog proxied to SupportFlow Agent Python `GET /api/channels`.
pub async fn agent_get_channel_catalog(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<serde_json::Value, String> {
    license.require_valid()?;
    runtime.channel_python_channels_get().await
}

#[tauri::command]
/// Channel connect/disconnect/save proxied to SupportFlow Agent Python `POST /api/channels`.
pub async fn agent_channel_action(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentChannelActionRequest,
) -> Result<serde_json::Value, String> {
    license.require_valid()?;
    let payload = serde_json::json!({
        "action": body.action,
        "channel": body.channel,
        "config": body.config,
    });
    runtime.channel_python_channels_post(payload).await
}

#[tauri::command]
/// Channel console APIs (WX QR login, WeWork contact sync) proxied to Python sidecar.
pub async fn agent_channel_console_api(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: AgentChannelConsoleApiRequest,
) -> Result<serde_json::Value, String> {
    license.require_valid()?;
    runtime
        .channel_console_api(&body.path, &body.method, body.body)
        .await
}

#[tauri::command]
/// List scheduled task summaries from runtime workspace.
pub async fn agent_list_tasks(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<Vec<AgentTaskSummary>, String> {
    license.require_valid()?;
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
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentLogsStatus, String> {
    license.require_valid()?;
    let (enabled, source) = runtime.logs_status().await?;
    Ok(AgentLogsStatus { enabled, source })
}

#[tauri::command]
/// Start background log tailing and push deltas through AGENT_LOG_STREAM event.
pub async fn agent_start_log_stream(
    app: AppHandle,
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentLogStreamState, String> {
    license.require_valid()?;
    let started = runtime.inner().clone().start_log_stream(app).await?;
    Ok(AgentLogStreamState { started })
}

#[tauri::command]
/// Stop background log tailing loop.
pub async fn agent_stop_log_stream(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
) -> Result<AgentLogStreamState, String> {
    license.require_valid()?;
    runtime.stop_log_stream().await;
    Ok(AgentLogStreamState { started: false })
}

#[tauri::command]
/// Read latest log lines with optional line limit.
pub async fn agent_read_logs(
    license: tauri::State<'_, LicenseStore>,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: Option<AgentReadLogsRequest>,
) -> Result<AgentReadLogsResult, String> {
    license.require_valid()?;
    let (source, content) = runtime.read_logs(body.and_then(|b| b.limit)).await?;

    Ok(AgentReadLogsResult { source, content })
}
