//! Multi-agent role bindings (planner / executor / reviewer).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// Fixed collaboration roles for workflow delegation (MVP).
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Planner,
    Executor,
    Reviewer,
}

impl AgentRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Executor => "executor",
            Self::Reviewer => "reviewer",
        }
    }
}

/// Binds a role to model/tools/skills for a workflow node or sub-session.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoleBinding {
    pub role: AgentRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt_suffix: Option<String>,
}

impl RoleBinding {
    pub fn planner_default() -> Self {
        Self {
            role: AgentRole::Planner,
            model: None,
            tools: vec![],
            skills: vec!["search-first".into()],
            system_prompt_suffix: Some("你是规划者：分解任务、列出步骤，不直接执行工具。".into()),
        }
    }

    pub fn executor_default() -> Self {
        Self {
            role: AgentRole::Executor,
            model: None,
            tools: vec![
                "read".into(),
                "write".into(),
                "bash".into(),
                "memory_search".into(),
            ],
            skills: vec![],
            system_prompt_suffix: Some("你是执行者：按计划调用工具完成任务。".into()),
        }
    }

    pub fn reviewer_default() -> Self {
        Self {
            role: AgentRole::Reviewer,
            model: None,
            tools: vec![],
            skills: vec![],
            system_prompt_suffix: Some(
                "你是审查者：汇总执行结果，指出风险与遗漏，输出最终答复。".into(),
            ),
        }
    }
}
