//! Workflow 编排运行时（MVP 设计阶段类型草稿，执行器见 T003）。

pub mod store;
pub mod types;

pub use store::{db_path_for_workspace, open_workflow_store, WorkflowStore};
pub use types::{
    BranchNodeConfig, DelayNodeConfig, HumanAndsignNodeConfig, NodeConfig, NodeKind, NodeState,
    RunStatus, StepRecord, StepStatus, Transition, WorkflowContext, WorkflowDefinition,
    WorkflowNode, WorkflowRun,
};
