//! Keyword search over workspace `memory/` files (fallback when DB memory not wired).

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::traits::{MemoryManager, MemorySearchHit};

pub struct FileKeywordMemoryManager {
    workspace: PathBuf,
    enable_knowledge: bool,
}

impl FileKeywordMemoryManager {
    pub fn new(workspace: PathBuf, enable_knowledge: bool) -> Self {
        Self {
            workspace,
            enable_knowledge,
        }
    }

    fn collect_memory_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let memory_md = self.workspace.join("MEMORY.md");
        if memory_md.is_file() {
            files.push(memory_md);
        }
        let memory_dir = self.workspace.join("memory");
        if memory_dir.is_dir() {
            walk_md(&memory_dir, &mut files);
        }
        if self.enable_knowledge {
            let knowledge_dir = self.workspace.join("knowledge");
            if knowledge_dir.is_dir() {
                walk_md(&knowledge_dir, &mut files);
            }
        }
        files
    }
}

fn walk_md(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_md(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

fn rel_path(workspace: &Path, file: &Path) -> String {
    file.strip_prefix(workspace)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| file.display().to_string())
}

#[async_trait]
impl MemoryManager for FileKeywordMemoryManager {
    fn workspace(&self) -> &Path {
        &self.workspace
    }

    async fn search(
        &self,
        query: &str,
        _user_id: Option<&str>,
        max_results: usize,
        min_score: f64,
    ) -> Result<Vec<MemorySearchHit>, String> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(vec![]);
        }

        let tokens: Vec<String> = query
            .split_whitespace()
            .map(|t| t.to_lowercase())
            .filter(|t| t.len() >= 2)
            .collect();
        if tokens.is_empty() {
            return Ok(vec![]);
        }

        let mut hits = Vec::new();

        for file in self.collect_memory_files() {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let rel = rel_path(&self.workspace, &file);
            for (i, line) in content.lines().enumerate() {
                let lower = line.to_lowercase();
                let matched = tokens.iter().filter(|t| lower.contains(t.as_str())).count();
                if matched == 0 {
                    continue;
                }
                let score = matched as f64 / tokens.len() as f64;
                if score < min_score {
                    continue;
                }
                let line_no = (i + 1) as u32;
                let snippet = if line.len() > 200 {
                    format!("{}...", &line[..200])
                } else {
                    line.to_string()
                };
                hits.push(MemorySearchHit {
                    path: rel.clone(),
                    start_line: line_no,
                    end_line: line_no,
                    score,
                    snippet,
                });
            }
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(max_results);
        Ok(hits)
    }
}
