//! Load workspace context files into the system prompt.

use std::path::Path;

use super::types::ContextFile;

const AGENT_FILE: &str = "AGENT.md";
const USER_FILE: &str = "USER.md";
const RULE_FILE: &str = "RULE.md";
const MEMORY_FILE: &str = "MEMORY.md";
const BOOTSTRAP_FILE: &str = "BOOTSTRAP.md";

fn is_template_placeholder(content: &str) -> bool {
    content.contains("<!-- template -->") || content.contains("请在此填写")
}

fn truncate_memory(content: &str, max_chars: usize) -> String {
    if content.chars().count() <= max_chars {
        return content.to_string();
    }
    let truncated: String = content.chars().take(max_chars).collect();
    format!("{truncated}\n\n[... MEMORY.md truncated for context ...]")
}

pub fn load_context_files(workspace_dir: &Path, files: Option<&[&str]>) -> Vec<ContextFile> {
    let default = [
        AGENT_FILE,
        USER_FILE,
        RULE_FILE,
        MEMORY_FILE,
        BOOTSTRAP_FILE,
    ];
    let names: Vec<&str> = files
        .map(|s| s.to_vec())
        .unwrap_or_else(|| default.to_vec());

    let mut out = Vec::new();
    for filename in names {
        let filepath = workspace_dir.join(filename);
        if !filepath.is_file() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&filepath) else {
            continue;
        };
        let content = content.trim().to_string();
        if content.is_empty() || is_template_placeholder(&content) {
            continue;
        }
        let content = if filename == MEMORY_FILE {
            truncate_memory(&content, 8000)
        } else {
            content
        };
        out.push(ContextFile {
            path: filename.to_string(),
            content,
        });
    }
    out
}
