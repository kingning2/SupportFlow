//! Workflow Tauri IPC（薄入口，逻辑在 `services/workflow/executor`）。

use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use typeshare::typeshare;

use crate::context::agent_runtime::AgentRuntime;
use crate::services::workflow::{open_workflow_store, WorkflowExecutor, WorkflowRun};

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStartRequest {
    pub definition_id: String,
    #[serde(default)]
    pub input: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStartResponse {
    pub run_id: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowGetRunRequest {
    pub run_id: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowResumeRequest {
    pub run_id: String,
}

/// 启动 workflow run（内置 `demo-linear` 等定义）。
#[tauri::command]
pub async fn workflow_start(
    app: AppHandle,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: WorkflowStartRequest,
) -> Result<WorkflowStartResponse, String> {
    let stack = runtime.bridge_stack.read().await.clone();
    let run_id = WorkflowExecutor::start(
        app,
        runtime.workspace.clone(),
        stack,
        &body.definition_id,
        body.input,
        body.session_id,
    )
    .await?;
    Ok(WorkflowStartResponse { run_id })
}

/// 查询 workflow run 快照（含 steps）。
#[tauri::command]
pub async fn workflow_get_run(
    runtime: State<'_, Arc<AgentRuntime>>,
    body: WorkflowGetRunRequest,
) -> Result<Option<WorkflowRun>, String> {
    let store = open_workflow_store(&runtime.workspace)?;
    store.load_run(&body.run_id)
}

/// 从持久化状态恢复执行（WaitingHuman / Paused / Running）。
#[tauri::command]
pub async fn workflow_resume(
    app: AppHandle,
    runtime: State<'_, Arc<AgentRuntime>>,
    body: WorkflowResumeRequest,
) -> Result<(), String> {
    let stack = runtime.bridge_stack.read().await.clone();
    WorkflowExecutor::resume(app, runtime.workspace.clone(), stack, &body.run_id).await
}
