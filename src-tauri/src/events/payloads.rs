//! 事件载荷类型（序列化契约，部分经 typeshare 导出到前端）。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppSession {
    pub current_language: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalLifecyclePayload {
    pub label: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalOpenPanelPayload {
    pub name: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeLogLevel {
    Info,
    Error,
    Warn,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeLogPayload {
    pub level: FeLogLevel,
    pub msg: String,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub source: String,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetail {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub source: String,
    pub file_path: String,
    pub base_dir: String,
    pub disable_model_invocation: bool,
}

/// Channel lifecycle push from Python sidecar (`channel/status-changed`).
#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatusChangedPayload {
    pub channel: String,
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr_code_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr_image: Option<String>,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolItem {
    pub name: String,
    pub description: String,
    pub is_mcp: bool,
}

/// SSE-style chunk emitted during `agent_send_message` (`agent/stream-chunk`).
#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamChunk {
    pub request_id: String,
    #[serde(rename = "type")]
    pub chunk_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time: Option<f64>,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunFinished {
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentLogStreamPayload {
    #[serde(rename = "type")]
    pub payload_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelProviderItem {
    /// `config.json` `bot_type` id (e.g. `deepseek`, `openai`).
    pub id: String,
    pub configured: bool,
    pub is_active: bool,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelProviderDetail {
    pub id: String,
    pub configured: bool,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_default: Option<String>,
    pub has_api_base: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_masked: Option<String>,
    pub models: Vec<String>,
    pub bot_type_value: String,
    pub editable: bool,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentConsoleState {
    pub session_id: String,
    pub workspace_dir: String,
    pub model_name: String,
    pub bot_type: String,
    pub providers: Vec<ModelProviderItem>,
    pub provider_details: Vec<ModelProviderDetail>,
    pub tools: Vec<ToolItem>,
    pub skills: Vec<SkillItem>,
    pub mcp_status: std::collections::HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_timeout: Option<u32>,
}
