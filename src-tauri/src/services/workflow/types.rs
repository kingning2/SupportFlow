//! Workflow 领域类型草稿（T001 设计评审，T002/T003 实现）。

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typeshare::typeshare;

use crate::services::agent::roles::{AgentRole, RoleBinding};

/// 工作流定义：静态节点图。
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub entry_node_id: String,
    pub nodes: Vec<WorkflowNode>,
    pub transitions: Vec<Transition>,
}

/// 图节点。
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNode {
    pub id: String,
    pub kind: NodeKind,
    pub label: Option<String>,
    pub config: NodeConfig,
}

/// MVP 首批节点类型。
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    AgentReply,
    ToolCall,
    HumanAndsign,
    Branch,
    Delay,
    DelegateToRole,
}

/// 节点类型相关配置（按 `kind` 选用对应字段）。
#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_reply: Option<AgentReplyNodeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<ToolCallNodeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_andsign: Option<HumanAndsignNodeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<BranchNodeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<DelayNodeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegate_to_role: Option<DelegateToRoleNodeConfig>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentReplyNodeConfig {
    pub prompt_template: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_filter: Option<Vec<String>>,
    #[serde(default)]
    pub clear_history: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<AgentRole>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DelegateToRoleNodeConfig {
    pub role: AgentRole,
    pub prompt_template: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<RoleBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u32>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallNodeConfig {
    pub tool_name: String,
    #[serde(default)]
    pub arguments: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_key: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HumanAndsignNodeConfig {
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u32>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BranchNodeConfig {
    /// 简单 MVP：按 context key 的布尔值选边；完整表达式 T003+。
    pub condition_key: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DelayNodeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_key: Option<String>,
}

/// 有向边；`condition` 为 `Some` 时表示分支出边（与 `branch` 节点配合）。
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// 一次工作流执行实例。
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRun {
    pub id: String,
    pub definition_id: String,
    pub status: RunStatus,
    pub current_node_id: Option<String>,
    pub context: WorkflowContext,
    pub steps: Vec<StepRecord>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Run 级执行状态。
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    WaitingHuman,
    Paused,
    Succeeded,
    Failed,
    Cancelled,
}

/// Run 级变量表（分支、节点输出、渠道元数据）。
#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowContext {
    #[serde(default)]
    pub vars: HashMap<String, Value>,
}

/// 单步执行记录。
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StepRecord {
    pub id: String,
    pub node_id: String,
    pub node_kind: NodeKind,
    pub status: StepStatus,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 节点单次执行状态（Step 内）。
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Queued,
    Active,
    Suspended,
    Completed,
    Failed,
}

/// 节点在 Run 内的瞬时状态（executor 内部使用，可选持久化）。
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    Idle,
    Queued,
    Active,
    Suspended,
    Completed,
    Failed,
}
