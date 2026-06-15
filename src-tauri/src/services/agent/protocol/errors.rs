//! Agent 流式执行错误类型。

use super::AgentCancelledError;

/// `Agent::run_stream` 与 rig 运行时的统一错误。
#[derive(Debug, thiserror::Error)]
pub enum RunStreamError {
    #[error("agent cancelled")]
    Cancelled(#[from] AgentCancelledError),
    #[error("{0}")]
    LlmFailed(String),
    #[error("{0}")]
    Other(String),
}
