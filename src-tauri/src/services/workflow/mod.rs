//! Workflow 编排运行时（MVP 设计阶段类型草稿，执行器见 T003）。

pub mod types;

pub use types::{
    BranchNodeConfig, DelayNodeConfig, HumanAndsignNodeConfig, NodeConfig, NodeKind, NodeState,
    RunStatus, StepRecord, StepStatus, Transition, WorkflowContext, WorkflowDefinition,
    WorkflowNode, WorkflowRun,
};
