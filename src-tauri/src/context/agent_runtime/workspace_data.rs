//! 工作区 memory 与 scheduler 任务列表（纯文件读写）。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::{AgentRuntime, RuntimeMemoryItem, RuntimeTaskItem};

fn collect_markdown_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
            continue;
        }
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext: &str| ext.eq_ignore_ascii_case("md"))
        {
            out.push(path);
        }
    }
    Ok(())
}

impl AgentRuntime {
    /// List memory markdown files from workspace and map metadata to UI rows.
    pub async fn list_memory_items(&self) -> Result<Vec<RuntimeMemoryItem>, String> {
        let mut files = Vec::new();
        let global = self.workspace.join("MEMORY.md");
        if global.is_file() {
            files.push(global);
        }
        collect_markdown_files(&self.workspace.join("memory"), &mut files)?;

        let mut rows = Vec::with_capacity(files.len());
        for path in files {
            let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or_default();
            let filename = path
                .strip_prefix(&self.workspace)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"));
            let item_type = if filename.contains("/dream") || filename.contains("dream/") {
                "dream".to_string()
            } else if filename.eq_ignore_ascii_case("memory.md") {
                "global".to_string()
            } else {
                "daily".to_string()
            };

            rows.push(RuntimeMemoryItem {
                filename,
                item_type,
                size: i32::try_from(meta.len()).unwrap_or(i32::MAX),
                updated_at: modified.to_string(),
            });
        }
        rows.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(rows)
    }

    /// Read one memory markdown file under workspace/memory.
    pub async fn read_memory_item(&self, filename: &str) -> Result<String, String> {
        let rel = filename.replace('\\', "/");
        if rel.contains("..") {
            return Err("invalid memory path".to_string());
        }
        let full = self.workspace.join(rel);
        if !full.starts_with(&self.workspace) {
            return Err("invalid memory path".to_string());
        }
        crate::utils::fs::read_to_string(full)
    }

    /// Parse scheduled tasks from config and map to task rows.
    pub async fn list_task_items(&self) -> Result<Vec<RuntimeTaskItem>, String> {
        let task_path = self.workspace.join("scheduler/tasks.json");
        if !task_path.exists() {
            return Ok(Vec::new());
        }
        let raw = crate::utils::fs::read_to_string(task_path)?;
        let value: serde_json::Value = crate::utils::json::from_str(&raw)?;
        let mut rows = Vec::new();
        let Some(obj) = value.get("tasks").and_then(|v| v.as_object()) else {
            return Ok(rows);
        };

        for (task_id, task) in obj {
            let name = task
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(task_id)
                .to_string();
            let enabled = task
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let next_run_at = task
                .get("next_run_at")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            rows.push(RuntimeTaskItem {
                id: task_id.clone(),
                name,
                enabled,
                next_run_at,
            });
        }
        rows.sort_by(|a, b| b.next_run_at.cmp(&a.next_run_at));
        Ok(rows)
    }
}
