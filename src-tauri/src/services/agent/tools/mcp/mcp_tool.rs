//! `agent/tools/mcp/mcp_tool.py`

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use super::client::{McpClient, McpToolSchema};
use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};

/// MCP-backed agent tool (`McpTool` in Python).
pub struct McpTool {
    client: Arc<McpClient>,
    server_name: String,
    tool_name: String,
    description: String,
    input_schema: Value,
}

impl McpTool {
    pub fn new(client: Arc<McpClient>, schema: McpToolSchema, server_name: String) -> Self {
        Self {
            client,
            server_name,
            tool_name: schema.name,
            description: schema.description,
            input_schema: schema.input_schema,
        }
    }

    pub fn from_parts(
        client: Arc<McpClient>,
        server_name: impl Into<String>,
        tool_name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
    ) -> Self {
        Self {
            client,
            server_name: server_name.into(),
            tool_name: tool_name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Back-compat alias used in early Step 7 stubs/tests.
pub type McpDynamicTool = McpTool;

#[async_trait]
impl AgentTool for McpTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    fn is_mcp(&self) -> bool {
        true
    }

    fn mcp_server_name(&self) -> Option<&str> {
        Some(&self.server_name)
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let args = match params {
            Value::Object(map) => Value::Object(map),
            other => other,
        };
        tracing::info!(
            server = %self.server_name,
            tool = %self.tool_name,
            "MCP tool execute"
        );
        let text = self.client.call_tool(&self.tool_name, args).await;
        ToolRunResult::success_text(text)
    }
}
