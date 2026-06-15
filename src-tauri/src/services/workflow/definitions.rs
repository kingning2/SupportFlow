//! 内置 workflow 定义（MVP 硬编码，后续可改为文件加载）。

use serde_json::json;

use super::types::{
    AgentReplyNodeConfig, NodeConfig, NodeKind, Transition, WorkflowDefinition, WorkflowNode,
};

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
        _ => None,
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
