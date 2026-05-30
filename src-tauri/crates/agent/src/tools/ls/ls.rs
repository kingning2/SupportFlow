//! `agent/tools/ls/ls.py`

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::tools::base_tool::{AgentTool, ToolRunResult};
use crate::tools::utils::path::is_cow_config_dir;
use crate::tools::utils::truncate::{format_size, truncate_head, DEFAULT_MAX_BYTES};
use crate::tools::workspace::WorkspaceToolConfig;

const DEFAULT_LIMIT: usize = 500;

pub struct LsTool {
    config: WorkspaceToolConfig,
    description: String,
}

impl LsTool {
    pub fn new(config: WorkspaceToolConfig) -> Self {
        let description = format!(
            "List directory contents. Returns entries sorted alphabetically, with '/' suffix for directories. Includes dotfiles. Output is truncated to {DEFAULT_LIMIT} entries or {}KB (whichever is hit first).",
            DEFAULT_MAX_BYTES / 1024
        );
        Self {
            config,
            description,
        }
    }
}

#[async_trait]
impl AgentTool for LsTool {
    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory to list. Relative paths use workspace directory."
                },
                "limit": {
                    "type": "integer",
                    "description": format!("Maximum number of entries to return (default: {DEFAULT_LIMIT})")
                }
            },
            "required": []
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".")
            .trim();
        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(DEFAULT_LIMIT);

        let absolute = self.config.resolve(path);

        if is_cow_config_dir(&absolute) {
            return ToolRunResult::error(
                "Error: Access denied. API keys and credentials must be accessed through the env_config tool only.",
            );
        }

        if !absolute.exists() {
            if !std::path::Path::new(path).is_absolute() && !path.starts_with('~') {
                return ToolRunResult::error(format!(
                    "Error: Path not found: {path}\nResolved to: {}\nHint: Relative paths are based on workspace ({}). For files outside workspace, use absolute paths.",
                    absolute.display(),
                    self.config.display_cwd().display()
                ));
            }
            return ToolRunResult::error(format!("Error: Path not found: {path}"));
        }

        if !absolute.is_dir() {
            return ToolRunResult::error(format!("Error: Not a directory: {path}"));
        }

        let mut entries: Vec<String> = match std::fs::read_dir(&absolute) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    let full = e.path();
                    if full.is_dir() {
                        Some(format!("{name}/"))
                    } else {
                        Some(name)
                    }
                })
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return ToolRunResult::error(format!(
                    "Error: Permission denied reading directory: {path}"
                ));
            }
            Err(e) => return ToolRunResult::error(format!("Error listing directory: {e}")),
        };

        entries.sort_by_key(|a| a.to_lowercase());

        let mut results = Vec::new();
        let mut entry_limit_reached = false;
        for entry in entries {
            if results.len() >= limit {
                entry_limit_reached = true;
                break;
            }
            results.push(entry);
        }

        if results.is_empty() {
            return ToolRunResult::success(json!({
                "message": "(empty directory)",
                "entries": [],
            }));
        }

        let raw_output = results.join("\n");
        let truncation = truncate_head(&raw_output, Some(999_999), None);
        let mut output = truncation.content.clone();
        let mut details = serde_json::Map::new();
        let mut notices = Vec::new();

        if entry_limit_reached {
            notices.push(format!(
                "{limit} entries limit reached. Use limit={} for more",
                limit * 2
            ));
            details.insert("entry_limit_reached".into(), json!(limit));
        }
        if truncation.truncated {
            notices.push(format!("{} limit reached", format_size(DEFAULT_MAX_BYTES)));
            details.insert("truncation".into(), truncation.to_value());
        }
        if !notices.is_empty() {
            output.push_str(&format!("\n\n[{}]", notices.join(". ")));
        }

        ToolRunResult::success(json!({
            "output": output,
            "entry_count": results.len(),
            "details": if details.is_empty() { Value::Null } else { Value::Object(details) },
        }))
    }
}
