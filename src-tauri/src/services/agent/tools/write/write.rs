//! `agent/tools/write/write.py`

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::workspace::WorkspaceToolConfig;

pub struct WriteTool {
    config: WorkspaceToolConfig,
}

impl WriteTool {
    pub fn new(config: WorkspaceToolConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AgentTool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates the file if it doesn't exist, overwrites if it does. Automatically creates parent directories. IMPORTANT: Single write should not exceed 10KB. For large files, create a skeleton first, then use edit to add content in chunks."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file to write (relative or absolute)" },
                "content": { "type": "string", "description": "Content to write to the file" }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if path.is_empty() {
            return ToolRunResult::error("Error: path parameter is required");
        }

        let absolute = self.config.resolve(path);

        if let Some(parent) = absolute.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolRunResult::error(format!("Error writing file: {e}"));
            }
        }

        match std::fs::write(&absolute, &content) {
            Ok(()) => {
                let bytes_written = content.len();
                ToolRunResult::success(json!({
                    "message": format!("Successfully wrote {bytes_written} bytes to {path}"),
                    "path": path,
                    "bytes_written": bytes_written,
                }))
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                ToolRunResult::error(format!("Error: Permission denied writing to {path}"))
            }
            Err(e) => ToolRunResult::error(format!("Error writing file: {e}")),
        }
    }
}
