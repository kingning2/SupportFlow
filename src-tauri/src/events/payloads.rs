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
