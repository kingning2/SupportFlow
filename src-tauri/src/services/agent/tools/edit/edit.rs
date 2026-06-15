//! `agent/tools/edit/edit.py`

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::utils::diff::{
    detect_line_ending, fuzzy_find_text, generate_diff_string, normalize_for_fuzzy_match,
    normalize_to_lf, restore_line_endings, strip_bom,
};
use crate::services::agent::tools::workspace::WorkspaceToolConfig;

pub struct EditTool {
    config: WorkspaceToolConfig,
}

impl EditTool {
    pub fn new(config: WorkspaceToolConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AgentTool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "通过精确匹配替换编辑文件；oldText 为空时追加到文件末尾。替换时 oldText 须与原文完全一致（含空白）。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file to edit" },
                "oldText": { "type": "string", "description": "Text to find and replace (empty to append)" },
                "newText": { "type": "string", "description": "New text" }
            },
            "required": ["path", "oldText", "newText"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let old_text = params
            .get("oldText")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let new_text = params
            .get("newText")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if path.is_empty() {
            return ToolRunResult::error("Error: path parameter is required");
        }

        let absolute = self.config.resolve(path);
        if !absolute.exists() {
            return ToolRunResult::error(format!("Error: File not found: {path}"));
        }

        let raw_content = match std::fs::read_to_string(&absolute) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                return ToolRunResult::error(format!(
                    "Error: File is not a valid text file (encoding error): {path}"
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return ToolRunResult::error(format!("Error: Permission denied accessing {path}"));
            }
            Err(e) => return ToolRunResult::error(format!("Error editing file: {e}")),
        };

        let (bom, content) = strip_bom(&raw_content);
        let original_ending = detect_line_ending(&content);
        let normalized_content = normalize_to_lf(&content);
        let normalized_old = normalize_to_lf(&old_text);
        let normalized_new = normalize_to_lf(&new_text);

        let (base_content, new_content) = if old_text.trim().is_empty() {
            let new_content =
                if !normalized_content.is_empty() && !normalized_content.ends_with('\n') {
                    format!("{normalized_content}\n{normalized_new}")
                } else {
                    format!("{normalized_content}{normalized_new}")
                };
            (normalized_content.clone(), new_content)
        } else {
            let match_result = fuzzy_find_text(&normalized_content, &normalized_old);
            if !match_result.found {
                return ToolRunResult::error(format!(
                    "Error: Could not find the exact text in {path}. The old text must match exactly including all whitespace and newlines."
                ));
            }

            let fuzzy_content = normalize_for_fuzzy_match(&normalized_content);
            let fuzzy_old = normalize_for_fuzzy_match(&normalized_old);
            let occurrences = fuzzy_content.matches(&fuzzy_old).count();
            if occurrences > 1 {
                return ToolRunResult::error(format!(
                    "Error: Found {occurrences} occurrences of the text in {path}. The text must be unique. Please provide more context to make it unique."
                ));
            }

            let base = match_result.content_for_replacement;
            let idx = match_result.index;
            let len = match_result.match_length;
            let new_content = format!("{}{}{}", &base[..idx], normalized_new, &base[idx + len..]);
            (base, new_content)
        };

        if base_content == new_content {
            return ToolRunResult::error(format!(
                "Error: No changes made to {path}. The replacement produced identical content. This might indicate an issue with special characters or the text not existing as expected."
            ));
        }

        let final_content = format!(
            "{}{}",
            bom,
            restore_line_endings(&new_content, &original_ending)
        );

        if let Err(e) = std::fs::write(&absolute, &final_content) {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return ToolRunResult::error(format!("Error: Permission denied accessing {path}"));
            }
            return ToolRunResult::error(format!("Error editing file: {e}"));
        }

        let diff_result = generate_diff_string(&base_content, &new_content);
        ToolRunResult::success(json!({
            "message": format!("Successfully replaced text in {path}"),
            "path": path,
            "diff": diff_result.diff,
            "first_changed_line": diff_result.first_changed_line,
        }))
    }
}
