//! 前后端 IPC 契约类型（`typeshare` → TypeScript）。
//!
//! 实现分布在各模块；本模块 re-export 便于 typeshare 扫描。

pub use crate::events::payloads::{
    AgentConsoleState, AgentRunFinished, AgentStreamChunk, AppSession, SkillItem, ToolItem,
};
