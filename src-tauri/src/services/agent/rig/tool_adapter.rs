//! 将现有 `AgentTool` 适配为 rig `ToolDyn`。

use std::sync::Arc;

use rig_core::completion::ToolDefinition;
use rig_core::tool::{ToolDyn, ToolError};
use rig_core::wasm_compat::WasmBoxedFuture;
use serde_json::Value;

use crate::services::agent::tools::{AgentTool, ToolRunResult};

/// 包装既有 `AgentTool`，供 rig Agent 工具循环调用。
pub struct LegacyAgentToolAdapter {
    inner: Arc<dyn AgentTool>,
}

impl LegacyAgentToolAdapter {
    /// 从共享工具实例创建适配器。
    pub fn new(inner: Arc<dyn AgentTool>) -> Self {
        Self { inner }
    }
}

fn tool_output_text(result: &ToolRunResult) -> String {
    match &result.result {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

impl ToolDyn for LegacyAgentToolAdapter {
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    fn definition<'a>(&'a self, _prompt: String) -> WasmBoxedFuture<'a, ToolDefinition> {
        let inner = self.inner.clone();
        Box::pin(async move {
            ToolDefinition {
                name: inner.name().to_string(),
                description: inner.description().to_string(),
                parameters: inner.input_schema(),
            }
        })
    }

    fn call<'a>(&'a self, args: String) -> WasmBoxedFuture<'a, Result<String, ToolError>> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let params = match serde_json::from_str::<Value>(&args) {
                Ok(v) => v,
                Err(_err) if args.trim() == "null" => Value::Object(Default::default()),
                Err(err) => return Err(ToolError::JsonError(err)),
            };
            let result = inner.execute(params).await;
            Ok(tool_output_text(&result))
        })
    }
}
