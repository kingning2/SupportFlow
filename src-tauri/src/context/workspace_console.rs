//! Workspace-backed console helpers (sessions index, knowledge, channel config).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

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
    fs::create_dir_all(workspace.join("sessions")).map_err(|e| e.to_string())
}

fn load_session_index(workspace: &Path) -> SessionIndexFile {
    let path = session_index_path(workspace);
    if !path.is_file() {
        return SessionIndexFile::default();
    }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_session_index(workspace: &Path, index: &SessionIndexFile) -> Result<(), String> {
    ensure_sessions_dir(workspace)?;
    let path = session_index_path(workspace);
    let text = serde_json::to_string_pretty(index).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
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
    let now = unix_ts_string();
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
                updated_at: unix_ts_string(),
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

fn unix_ts_string() -> String {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
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
    let svc = agent::knowledge::KnowledgeService::new(workspace);
    if !svc.knowledge_dir().is_dir() {
        fs::create_dir_all(svc.knowledge_dir()).map_err(|e| e.to_string())?;
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
    let svc = agent::knowledge::KnowledgeService::new(workspace);
    Ok(svc.read_file(rel_path)?.content)
}

/// Build a minimal knowledge graph from markdown cross-links.
pub fn build_knowledge_graph(workspace: &Path) -> Result<KnowledgeGraphData, String> {
    let graph = agent::knowledge::KnowledgeService::new(workspace).build_graph();
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

struct ChannelMeta {
    id: &'static str,
    label: &'static str,
}

const KNOWN_CHANNELS: &[ChannelMeta] = &[
    ChannelMeta {
        id: "feishu",
        label: "飞书",
    },
    ChannelMeta {
        id: "dingtalk",
        label: "钉钉",
    },
    ChannelMeta {
        id: "weixin",
        label: "微信",
    },
    ChannelMeta {
        id: "wx",
        label: "微信 (itchat)",
    },
    ChannelMeta {
        id: "wework",
        label: "企业微信",
    },
    ChannelMeta {
        id: "wechatmp",
        label: "微信公众号",
    },
    ChannelMeta {
        id: "wechatmp_service",
        label: "微信公众号（服务号）",
    },
    ChannelMeta {
        id: "wechatcom_app",
        label: "企业微信应用",
    },
    ChannelMeta {
        id: "wecom_bot",
        label: "企微机器人",
    },
    ChannelMeta {
        id: "qq",
        label: "QQ",
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

/// Channels that belong to the Python server/CLI stack, not the Tauri desktop app.
const DESKTOP_EXCLUDED_CHANNELS: &[&str] = &["web", "terminal"];

fn is_desktop_listed_channel(name: &str) -> bool {
    !DESKTOP_EXCLUDED_CHANNELS.contains(&name)
}

fn parse_channel_types(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::String(s) => s
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// Active channels from `channel_type` in config (no Rust channel catalog).
pub fn list_channels_from_config(config_path: &Path) -> Result<Vec<ChannelRow>, String> {
    let root = models::provider_catalog::read_config_root(config_path)?;
    let names = parse_channel_types(root.get("channel_type").unwrap_or(&serde_json::Value::Null));
    Ok(names
        .into_iter()
        .filter(|name| is_desktop_listed_channel(name))
        .map(|name| ChannelRow {
            label: channel_label(&name),
            active: true,
            name,
        })
        .collect())
}
