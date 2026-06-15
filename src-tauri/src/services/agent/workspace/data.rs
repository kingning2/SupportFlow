//! Workspace-backed console helpers (sessions index, knowledge, channel config).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub id: String,
    pub title: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeFileRow {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGraphNodeRow {
    pub id: String,
    pub label: String,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGraphLinkRow {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGraphData {
    pub nodes: Vec<KnowledgeGraphNodeRow>,
    pub links: Vec<KnowledgeGraphLinkRow>,
}

#[derive(Debug, Clone)]
pub struct ChannelRow {
    pub name: String,
    pub active: bool,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionIndexEntry {
    id: String,
    title: String,
    updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SessionIndexFile {
    sessions: Vec<SessionIndexEntry>,
}

fn session_index_path(workspace: &Path) -> PathBuf {
    workspace.join("sessions").join("index.json")
}

fn ensure_sessions_dir(workspace: &Path) -> Result<(), String> {
    crate::utils::fs::create_dir_all(workspace.join("sessions"))
}

fn load_session_index(workspace: &Path) -> SessionIndexFile {
    let path = session_index_path(workspace);
    if !path.is_file() {
        return SessionIndexFile::default();
    }
    let raw = crate::utils::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_session_index(workspace: &Path, index: &SessionIndexFile) -> Result<(), String> {
    ensure_sessions_dir(workspace)?;
    let path = session_index_path(workspace);
    let text = crate::utils::json::to_string_pretty(index)?;
    crate::utils::fs::write(path, text)
}

/// Upsert one session row in `workspace/sessions/index.json`.
pub fn upsert_session_index(
    workspace: &Path,
    session_id: &str,
    title: Option<&str>,
) -> Result<(), String> {
    if session_id.is_empty() {
        return Ok(());
    }
    let now = crate::utils::date::unix_timestamp_string();
    let mut index = load_session_index(workspace);
    if let Some(row) = index.sessions.iter_mut().find(|s| s.id == session_id) {
        if let Some(t) = title {
            if !t.trim().is_empty() {
                row.title = t.trim().to_string();
            }
        }
        row.updated_at = now;
    } else {
        index.sessions.push(SessionIndexEntry {
            id: session_id.to_string(),
            title: title
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("New Chat")
                .to_string(),
            updated_at: now,
        });
    }
    index
        .sessions
        .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    save_session_index(workspace, &index)
}

/// List sessions from workspace index; always includes `current_session_id` when set.
pub fn list_session_summaries(
    workspace: &Path,
    current_session_id: Option<&str>,
) -> Result<Vec<SessionRow>, String> {
    let mut index = load_session_index(workspace);
    if let Some(current) = current_session_id {
        if !current.is_empty() && !index.sessions.iter().any(|s| s.id == current) {
            index.sessions.push(SessionIndexEntry {
                id: current.to_string(),
                title: "New Chat".into(),
                updated_at: crate::utils::date::unix_timestamp_string(),
            });
        }
    }
    index
        .sessions
        .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(index
        .sessions
        .into_iter()
        .map(|s| SessionRow {
            id: s.id,
            title: s.title,
            updated_at: s.updated_at,
        })
        .collect())
}

#[allow(dead_code)]
fn resolve_under_workspace(workspace: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel = rel.replace('\\', "/");
    if rel.is_empty() || rel.contains("..") {
        return Err("invalid path".into());
    }
    let full = workspace.join(rel);
    for component in full.components() {
        if component == std::path::Component::ParentDir {
            return Err("invalid path".into());
        }
    }
    Ok(full)
}

/// Flat list of markdown files under `workspace/knowledge/`.
pub fn list_knowledge_files(workspace: &Path) -> Result<Vec<KnowledgeFileRow>, String> {
    let svc = crate::services::agent::knowledge::KnowledgeService::new(workspace);
    if !svc.knowledge_dir().is_dir() {
        crate::utils::fs::create_dir_all(svc.knowledge_dir())?;
        return Ok(Vec::new());
    }
    Ok(svc
        .list_files_flat()?
        .into_iter()
        .map(|(path, title)| KnowledgeFileRow { path, title })
        .collect())
}

/// Read one knowledge markdown file by path relative to `knowledge/`.
pub fn read_knowledge_file(workspace: &Path, rel_path: &str) -> Result<String, String> {
    let svc = crate::services::agent::knowledge::KnowledgeService::new(workspace);
    Ok(svc.read_file(rel_path)?.content)
}

/// Build a minimal knowledge graph from markdown cross-links.
pub fn build_knowledge_graph(workspace: &Path) -> Result<KnowledgeGraphData, String> {
    let graph = crate::services::agent::knowledge::KnowledgeService::new(workspace).build_graph();
    Ok(KnowledgeGraphData {
        nodes: graph
            .nodes
            .into_iter()
            .map(|n| KnowledgeGraphNodeRow {
                id: n.id,
                label: n.label,
                category: n.category,
            })
            .collect(),
        links: graph
            .links
            .into_iter()
            .map(|l| KnowledgeGraphLinkRow {
                source: l.source,
                target: l.target,
            })
            .collect(),
    })
}

/// Remove one knowledge file by relative path and clean up the memory index.
pub fn remove_knowledge_file(workspace: &Path, rel_path: &str) -> Result<(), String> {
    let svc = crate::services::agent::knowledge::KnowledgeService::new(workspace);
    svc.remove_file(rel_path)?;

    let db_path = workspace.join("memory/long-term/index.db");
    if db_path.is_file() {
        match crate::services::agent::memory::MemoryStorage::open(&db_path) {
            Ok(storage) => {
                let _: Result<(), String> =
                    storage.delete_by_path(&format!("knowledge/{rel_path}"));
            }
            Err(e) => {
                crate::log_warn!("clean up memory index after remove: {e}");
            }
        }
    }
    Ok(())
}

struct ChannelMeta {
    id: &'static str,
    label: &'static str,
}

const KNOWN_CHANNELS: &[ChannelMeta] = &[
    ChannelMeta {
        id: "wework",
        label: "企微个人号",
    },
    ChannelMeta {
        id: "terminal",
        label: "终端",
    },
];

fn channel_label(id: &str) -> String {
    KNOWN_CHANNELS
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.label.to_string())
        .unwrap_or_else(|| id.to_string())
}

/// Active channels from `channel_type` in config (no Rust channel catalog).
pub fn list_channels_from_config(config_path: &Path) -> Result<Vec<ChannelRow>, String> {
    let root = crate::config::provider_catalog::read_config_root(config_path)?;
    let names = crate::utils::channel::parse_desktop_channel_types(root.get("channel_type"));
    Ok(names
        .into_iter()
        .map(|name| ChannelRow {
            label: channel_label(&name),
            active: true,
            name,
        })
        .collect())
}
