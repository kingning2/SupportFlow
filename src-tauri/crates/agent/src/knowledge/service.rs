//! `agent/knowledge/service.py` — list / read / graph / upload.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::ingest::{ingest_files, trigger_memory_sync, IngestBatchResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFileEntry {
    pub name: String,
    pub title: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTreeNode {
    pub dir: String,
    pub files: Vec<KnowledgeFileEntry>,
    #[serde(default)]
    pub children: Vec<KnowledgeTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTreeStats {
    pub pages: usize,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTree {
    #[serde(default)]
    pub root_files: Vec<KnowledgeFileEntry>,
    pub tree: Vec<KnowledgeTreeNode>,
    pub stats: KnowledgeTreeStats,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeReadResult {
    pub content: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphNode {
    pub id: String,
    pub label: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphLink {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeGraphNode>,
    pub links: Vec<KnowledgeGraphLink>,
}

pub struct KnowledgeService {
    workspace_root: PathBuf,
    knowledge_dir: PathBuf,
}

impl KnowledgeService {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let workspace_root = workspace_root.into();
        let knowledge_dir = workspace_root.join("knowledge");
        Self {
            workspace_root,
            knowledge_dir,
        }
    }

    pub fn knowledge_dir(&self) -> &Path {
        &self.knowledge_dir
    }

    pub fn list_tree(&self, knowledge_enabled: bool) -> KnowledgeTree {
        if !self.knowledge_dir.is_dir() {
            return KnowledgeTree {
                root_files: Vec::new(),
                tree: Vec::new(),
                stats: KnowledgeTreeStats { pages: 0, size: 0 },
                enabled: knowledge_enabled,
            };
        }

        let mut stats = KnowledgeTreeStats { pages: 0, size: 0 };
        let (root_files, tree) = self.scan_dir(&self.knowledge_dir, &mut stats, true);
        KnowledgeTree {
            root_files,
            tree,
            stats,
            enabled: knowledge_enabled,
        }
    }

    fn scan_dir(
        &self,
        dir_path: &Path,
        stats: &mut KnowledgeTreeStats,
        is_root: bool,
    ) -> (Vec<KnowledgeFileEntry>, Vec<KnowledgeTreeNode>) {
        let mut files = Vec::new();
        let mut children = Vec::new();

        let Ok(entries) = fs::read_dir(dir_path) else {
            return (files, children);
        };

        let mut names: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        names.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for full in names {
            let name = full
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if name.starts_with('.') || name == "_sources" {
                continue;
            }
            if full.is_dir() {
                let (sub_files, sub_children) = self.scan_dir(&full, stats, false);
                children.push(KnowledgeTreeNode {
                    dir: name,
                    files: sub_files,
                    children: sub_children,
                });
            } else if name.ends_with(".md") {
                let size = fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
                if !is_root {
                    stats.pages += 1;
                    stats.size += size;
                }
                let title = title_from_md(&full, &name.replace(".md", ""));
                files.push(KnowledgeFileEntry { name, title, size });
            }
        }
        (files, children)
    }

    pub fn read_file(&self, rel_path: &str) -> Result<KnowledgeReadResult, String> {
        let full = resolve_under_knowledge(&self.knowledge_dir, rel_path)?;
        if !full.is_file() {
            return Err(format!("file not found: {rel_path}"));
        }
        let content = fs::read_to_string(&full).map_err(|e| e.to_string())?;
        Ok(KnowledgeReadResult {
            content,
            path: rel_path.trim_start_matches('/').replace('\\', "/"),
        })
    }

    pub fn build_graph(&self) -> KnowledgeGraph {
        if !self.knowledge_dir.is_dir() {
            return KnowledgeGraph {
                nodes: Vec::new(),
                links: Vec::new(),
            };
        }

        let link_re = Regex::new(r"\[([^\]]*)\]\(([^)]+\.md)\)").expect("link regex");
        let mut nodes: std::collections::HashMap<String, KnowledgeGraphNode> =
            std::collections::HashMap::new();
        let mut links = Vec::new();

        for entry in walkdir::WalkDir::new(&self.knowledge_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let md_file = entry.path();
            let rel = md_file
                .strip_prefix(&self.knowledge_dir)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            if rel == "index.md" || rel == "log.md" {
                continue;
            }
            if !rel.ends_with(".md") {
                continue;
            }
            let parts: Vec<_> = rel.split('/').collect();
            let category = parts.first().copied().unwrap_or("root").to_string();
            let mut title = md_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("doc")
                .replace('-', " ");
            if let Ok(content) = fs::read_to_string(md_file) {
                if let Some(first) = content.lines().find(|l| !l.trim().is_empty()) {
                    if let Some(rest) = first.trim().strip_prefix("# ") {
                        if !rest.is_empty() {
                            title = rest.to_string();
                        }
                    }
                }
                for cap in link_re.captures_iter(&content) {
                    let link_target = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                    let resolved = md_file.parent().unwrap_or(md_file).join(link_target);
                    let Ok(resolved) = resolved.canonicalize() else {
                        continue;
                    };
                    let Ok(target_rel) = resolved.strip_prefix(&self.knowledge_dir) else {
                        continue;
                    };
                    let target_rel = target_rel.to_string_lossy().replace('\\', "/");
                    if target_rel != rel {
                        links.push(KnowledgeGraphLink {
                            source: rel.clone(),
                            target: target_rel,
                        });
                    }
                }
            }
            nodes.insert(
                rel.clone(),
                KnowledgeGraphNode {
                    id: rel,
                    label: title,
                    category,
                },
            );
        }

        let valid: std::collections::HashSet<_> = nodes.keys().cloned().collect();
        links.retain(|l| valid.contains(&l.source) && valid.contains(&l.target));
        let mut seen = std::collections::HashSet::new();
        links.retain(|l| {
            let key = if l.source < l.target {
                (l.source.clone(), l.target.clone())
            } else {
                (l.target.clone(), l.source.clone())
            };
            seen.insert(key)
        });

        KnowledgeGraph {
            nodes: nodes.into_values().collect(),
            links,
        }
    }

    pub async fn ingest_upload(
        &self,
        files: Vec<(String, Vec<u8>)>,
        category: &str,
        sync_memory: bool,
        knowledge_enabled: bool,
        models_config: &models::ModelsConfig,
    ) -> Result<IngestBatchResult, String> {
        if files.is_empty() {
            return Err("no files provided".into());
        }
        let mut result = ingest_files(&self.knowledge_dir, &files, category, knowledge_enabled);
        if sync_memory && result.count > 0 {
            result.memory_synced = trigger_memory_sync(&self.workspace_root, models_config).await;
        }
        Ok(result)
    }

    /// Flat file list (for IPC / legacy callers).
    pub fn list_files_flat(&self) -> Result<Vec<(String, String)>, String> {
        let tree = self.list_tree(true);
        let mut out = Vec::new();
        for f in &tree.root_files {
            out.push((f.name.clone(), f.title.clone()));
        }
        fn walk(nodes: &[KnowledgeTreeNode], prefix: &str, out: &mut Vec<(String, String)>) {
            for node in nodes {
                let dir_prefix = if prefix.is_empty() {
                    node.dir.clone()
                } else {
                    format!("{}/{}", prefix, node.dir)
                };
                for f in &node.files {
                    let path = format!("{}/{}", dir_prefix, f.name);
                    out.push((path, f.title.clone()));
                }
                walk(&node.children, &dir_prefix, out);
            }
        }
        walk(&tree.tree, "", &mut out);
        out.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(out)
    }

    /// Remove one knowledge file by relative path.
    pub fn remove_file(&self, rel_path: &str) -> Result<(), String> {
        let full = resolve_under_knowledge(&self.knowledge_dir, rel_path)?;
        fs::remove_file(full).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn resolve_under_knowledge(knowledge_root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel = rel.trim_start_matches('/').replace('\\', "/");
    if rel.is_empty() || rel.contains("..") {
        return Err("invalid path".into());
    }
    let full = knowledge_root.join(&rel);
    for component in full.components() {
        if component == std::path::Component::ParentDir {
            return Err("invalid path".into());
        }
    }
    let knowledge_root = knowledge_root
        .canonicalize()
        .unwrap_or_else(|_| knowledge_root.to_path_buf());
    let full = full
        .canonicalize()
        .map_err(|_| format!("file not found: {rel}"))?;
    if !full.starts_with(&knowledge_root) {
        return Err("path outside knowledge dir".into());
    }
    Ok(full)
}

fn title_from_md(path: &Path, fallback: &str) -> String {
    let Ok(raw) = fs::read_to_string(path) else {
        return fallback.to_string();
    };
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# ") {
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }
    fallback.to_string()
}
