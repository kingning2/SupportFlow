//! 内置 workflow 定义（MVP 硬编码，后续可改为文件加载）。

use serde_json::json;

use super::types::{
    AgentReplyNodeConfig, DelegateToRoleNodeConfig, NodeConfig, NodeKind, Transition,
    WorkflowDefinition, WorkflowNode,
};
use crate::services::agent::roles::{AgentRole, RoleBinding};

/// 三步线性 demo：`agent_reply` × 3。
pub fn demo_linear_definition() -> WorkflowDefinition {
    WorkflowDefinition {
        id: "demo-linear".into(),
        name: "Demo Linear Flow".into(),
        version: 1,
        entry_node_id: "step-1".into(),
        nodes: vec![
            WorkflowNode {
                id: "step-1".into(),
                kind: NodeKind::AgentReply,
                label: Some("首轮回复".into()),
                config: NodeConfig {
                    agent_reply: Some(AgentReplyNodeConfig {
                        prompt_template: "请用一句话回应用户输入：{{input}}".into(),
                        skill_filter: None,
                        clear_history: true,
                        output_key: Some("reply_1".into()),
                        role: None,
                    }),
                    ..Default::default()
                },
            },
            WorkflowNode {
                id: "step-2".into(),
                kind: NodeKind::AgentReply,
                label: Some("补充说明".into()),
                config: NodeConfig {
                    agent_reply: Some(AgentReplyNodeConfig {
                        prompt_template: "基于上一轮结果「{{reply_1}}」，再补充一句建议。".into(),
                        skill_filter: None,
                        clear_history: false,
                        output_key: Some("reply_2".into()),
                        role: None,
                    }),
                    ..Default::default()
                },
            },
            WorkflowNode {
                id: "step-3".into(),
                kind: NodeKind::AgentReply,
                label: Some("总结".into()),
                config: NodeConfig {
                    agent_reply: Some(AgentReplyNodeConfig {
                        prompt_template: "将以下内容总结为一句结束语：{{reply_1}} / {{reply_2}}"
                            .into(),
                        skill_filter: None,
                        clear_history: false,
                        output_key: Some("summary".into()),
                        role: None,
                    }),
                    ..Default::default()
                },
            },
        ],
        transitions: vec![
            Transition {
                from: "step-1".into(),
                to: "step-2".into(),
                condition: None,
                label: None,
            },
            Transition {
                from: "step-2".into(),
                to: "step-3".into(),
                condition: None,
                label: None,
            },
        ],
    }
}

/// 按 id 解析内置定义。
pub fn builtin_definition(definition_id: &str) -> Option<WorkflowDefinition> {
    match definition_id {
        "demo-linear" => Some(demo_linear_definition()),
        "demo-multi-agent" => Some(demo_multi_agent_definition()),
        _ => None,
    }
}

/// Planner → executor → reviewer demo (role delegation, MVP).
pub fn demo_multi_agent_definition() -> WorkflowDefinition {
    WorkflowDefinition {
        id: "demo-multi-agent".into(),
        name: "Demo Multi-Agent Flow".into(),
        version: 1,
        entry_node_id: "plan".into(),
        nodes: vec![
            WorkflowNode {
                id: "plan".into(),
                kind: NodeKind::DelegateToRole,
                label: Some("Planner".into()),
                config: NodeConfig {
                    delegate_to_role: Some(DelegateToRoleNodeConfig {
                        role: AgentRole::Planner,
                        prompt_template:
                            "分析用户输入「{{input}}」，输出 3 步以内的执行计划（纯文本）。".into(),
                        binding: Some(RoleBinding::planner_default()),
                        output_key: Some("plan".into()),
                        timeout_secs: Some(120),
                    }),
                    ..Default::default()
                },
            },
            WorkflowNode {
                id: "execute".into(),
                kind: NodeKind::DelegateToRole,
                label: Some("Executor".into()),
                config: NodeConfig {
                    delegate_to_role: Some(DelegateToRoleNodeConfig {
                        role: AgentRole::Executor,
                        prompt_template: "按计划执行：{{plan}}。可调用工具，输出执行结果。".into(),
                        binding: Some(RoleBinding::executor_default()),
                        output_key: Some("execution".into()),
                        timeout_secs: Some(300),
                    }),
                    ..Default::default()
                },
            },
            WorkflowNode {
                id: "review".into(),
                kind: NodeKind::DelegateToRole,
                label: Some("Reviewer".into()),
                config: NodeConfig {
                    delegate_to_role: Some(DelegateToRoleNodeConfig {
                        role: AgentRole::Reviewer,
                        prompt_template:
                            "审查计划「{{plan}}」与执行结果「{{execution}}」，输出面向用户的最终答复。"
                                .into(),
                        binding: Some(RoleBinding::reviewer_default()),
                        output_key: Some("final_reply".into()),
                        timeout_secs: Some(120),
                    }),
                    ..Default::default()
                },
            },
        ],
        transitions: vec![
            Transition {
                from: "plan".into(),
                to: "execute".into(),
                condition: None,
                label: None,
            },
            Transition {
                from: "execute".into(),
                to: "review".into(),
                condition: None,
                label: None,
            },
        ],
    }
}

/// 将 demo 定义的输入写入 context vars。
pub fn seed_context_input(input: &serde_json::Value) -> super::types::WorkflowContext {
    let mut ctx = super::types::WorkflowContext::default();
    ctx.vars.insert("input".into(), input.clone());
    if let Some(s) = input.as_str() {
        ctx.vars.insert("input_text".into(), json!(s));
    }
    ctx
}
