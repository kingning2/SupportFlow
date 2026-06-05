//! `agent/tools/memory/memory_get.py`

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::Value;

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::memory::traits::MemoryManager;

pub struct MemoryGetTool {
    memory: std::sync::Arc<dyn MemoryManager>,
    description: String,
}

impl MemoryGetTool {
    pub fn new(memory: std::sync::Arc<dyn MemoryManager>, enable_knowledge: bool) -> Self {
        let description = if enable_knowledge {
            "Read specific content from memory or knowledge files. Use this to get full context from a memory file, knowledge page, or specific line range."
        } else {
            "Read specific content from memory files. Use this to get full context from a memory file or specific line range."
        }
        .to_string();
        Self {
            memory,
            description,
        }
    }

    fn resolve_memory_path(workspace: &Path, path: &str) -> PathBuf {
        let path = path.trim();
        if path.starts_with("memory/") || path.starts_with("knowledge/") || path.starts_with('/') {
            workspace.join(path.trim_start_matches('/'))
        } else if path == "MEMORY.md" {
            workspace.join("MEMORY.md")
        } else {
            workspace.join("memory").join(path)
        }
    }

    fn is_inside_workspace(workspace: &Path, file: &Path) -> bool {
        let Ok(ws) = workspace.canonicalize() else {
            return false;
        };
        let Ok(fp) = file.canonicalize() else {
            return false;
        };
        fp.starts_with(&ws)
    }
}

#[async_trait]
impl AgentTool for MemoryGetTool {
    fn name(&self) -> &str {
        "memory_get"
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path (e.g. MEMORY.md, memory/2026-01-01.md, knowledge/foo.md)"
                },
                "start_line": { "type": "integer", "description": "Starting line (default 1)" },
                "num_lines": { "type": "integer", "description": "Number of lines to read" }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let start_line = params
            .get("start_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
            .max(1) as usize;
        let num_lines = params
            .get("num_lines")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        if path.is_empty() {
            return ToolRunResult::error("Error: path parameter is required");
        }

        let workspace = self.memory.workspace();
        let file_path = Self::resolve_memory_path(workspace, path);

        if !Self::is_inside_workspace(workspace, &file_path) {
            return ToolRunResult::error("Error: Access denied: path outside workspace");
        }

        if !file_path.exists() {
            return ToolRunResult::error(format!("Error: File not found: {path}"));
        }

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => return ToolRunResult::error(format!("Error reading memory file: {e}")),
        };

        let lines: Vec<&str> = content.split('\n').collect();
        let total_lines = lines.len();
        let start_idx = start_line.saturating_sub(1);

        let selected: Vec<&str> = if let Some(n) = num_lines {
            lines.iter().skip(start_idx).take(n).copied().collect()
        } else {
            lines.iter().skip(start_idx).copied().collect()
        };

        let shown = selected.len();
        let end_line = if shown == 0 {
            start_line
        } else {
            start_line + shown - 1
        };

        let output = format!(
            "File: {path}\nLines: {start_line}-{end_line} (total: {total_lines})\n\n{}",
            selected.join("\n")
        );

        ToolRunResult::success_text(output)
    }
}
