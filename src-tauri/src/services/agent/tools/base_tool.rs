//! `agent/tools/base_tool.py`

use async_trait::async_trait;
use serde_json::Value;

/// Tool decision stage (`ToolStage` in Python).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStage {
    PreProcess,
    PostProcess,
}

/// Tool execution result (`ToolResult` in Python).
#[derive(Debug, Clone)]
pub struct ToolRunResult {
    pub status: String,
    pub result: Value,
    pub execution_time: f64,
}

impl ToolRunResult {
    pub fn success(result: Value) -> Self {
        Self {
            status: "success".into(),
            result,
            execution_time: 0.0,
        }
    }

    pub fn success_text(result: impl Into<String>) -> Self {
        Self::success(Value::String(result.into()))
    }

    pub fn error(result: impl Into<String>) -> Self {
        Self {
            status: "error".into(),
            result: Value::String(result.into()),
            execution_time: 0.0,
        }
    }

    pub fn fail_value(result: Value) -> Self {
        Self {
            status: "error".into(),
            result,
            execution_time: 0.0,
        }
    }

    pub fn critical_error(result: impl Into<String>) -> Self {
        Self {
            status: "critical_error".into(),
            result: Value::String(result.into()),
            execution_time: 0.0,
        }
    }
}

/// Runnable agent tool (`BaseTool` in Python).
#[async_trait]
pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn stage(&self) -> ToolStage {
        ToolStage::PreProcess
    }

    /// True for tools loaded from an MCP server (`McpTool` in Python).
    fn is_mcp(&self) -> bool {
        false
    }

    /// MCP server name when [`is_mcp`](Self::is_mcp) is true.
    fn mcp_server_name(&self) -> Option<&str> {
        None
    }

    fn json_schema(&self) -> Value {
        serde_json::json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": self.input_schema(),
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult;
}
