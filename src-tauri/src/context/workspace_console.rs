//! Workspace-backed console helpers (sessions index, knowledge, channel config).

use std::collections::{HashMap, HashSet};
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

fn resolve_knowledge_file(workspace: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel = rel.trim_start_matches('/');
    let knowledge_root = workspace.join("knowledge");
    let full = resolve_under_workspace(&knowledge_root, rel)?;
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

fn collect_knowledge_md_files(dir: &Path, knowledge_root: &Path, out: &mut Vec<KnowledgeFileRow>) {
    if !dir.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.') || n == "_sources")
            {
                continue;
            }
            collect_knowledge_md_files(&path, knowledge_root, out);
            continue;
        }
        if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            let rel = path
                .strip_prefix(knowledge_root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");
            let title = title_from_md(&path, stem);
            out.push(KnowledgeFileRow { path: rel, title });
        }
    }
}

/// Flat list of markdown files under `workspace/knowledge/`.
pub fn list_knowledge_files(workspace: &Path) -> Result<Vec<KnowledgeFileRow>, String> {
    let knowledge_root = workspace.join("knowledge");
    if !knowledge_root.is_dir() {
        fs::create_dir_all(&knowledge_root).map_err(|e| e.to_string())?;
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_knowledge_md_files(&knowledge_root, &knowledge_root, &mut files);
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

/// Read one knowledge markdown file by path relative to `knowledge/`.
pub fn read_knowledge_file(workspace: &Path, rel_path: &str) -> Result<String, String> {
    let full = resolve_knowledge_file(workspace, rel_path)?;
    if !full.is_file() {
        return Err(format!("file not found: {rel_path}"));
    }
    fs::read_to_string(full).map_err(|e| e.to_string())
}

fn extract_md_links(content: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("](") {
        let after = &rest[start + 2..];
        let Some(end) = after.find(')') else {
            break;
        };
        let target = after[..end].trim();
        if target.ends_with(".md") {
            targets.push(target.to_string());
        }
        rest = &after[end + 1..];
    }
    targets
}

/// Build a minimal knowledge graph from markdown cross-links.
pub fn build_knowledge_graph(workspace: &Path) -> Result<KnowledgeGraphData, String> {
    let knowledge_root = workspace.join("knowledge");
    if !knowledge_root.is_dir() {
        return Ok(KnowledgeGraphData {
            nodes: Vec::new(),
            links: Vec::new(),
        });
    }

    let files = list_knowledge_files(workspace)?;
    let mut nodes: HashMap<String, KnowledgeGraphNodeRow> = HashMap::new();
    let mut links = Vec::new();

    for file in &files {
        if file.path == "index.md" || file.path == "log.md" {
            continue;
        }
        let category = file.path.split('/').next().unwrap_or("root").to_string();
        nodes.insert(
            file.path.clone(),
            KnowledgeGraphNodeRow {
                id: file.path.clone(),
                label: file.title.clone(),
                category,
            },
        );
    }

    for file in &files {
        if file.path == "index.md" || file.path == "log.md" {
            continue;
        }
        let full = knowledge_root.join(&file.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let Ok(content) = fs::read_to_string(&full) else {
            continue;
        };
        for target in extract_md_links(&content) {
            let resolved = full
                .parent()
                .unwrap_or(&knowledge_root)
                .join(&target.replace('/', std::path::MAIN_SEPARATOR_STR));
            let Ok(resolved) = resolved.canonicalize() else {
                continue;
            };
            let Ok(target_rel) = resolved.strip_prefix(&knowledge_root) else {
                continue;
            };
            let target_rel = target_rel.to_string_lossy().replace('\\', "/");
            if target_rel != file.path && nodes.contains_key(&target_rel) {
                links.push(KnowledgeGraphLinkRow {
                    source: file.path.clone(),
                    target: target_rel,
                });
            }
        }
    }

    let mut seen = HashSet::new();
    links.retain(|l| {
        let key = if l.source < l.target {
            (l.source.clone(), l.target.clone())
        } else {
            (l.target.clone(), l.source.clone())
        };
        seen.insert(key)
    });

    Ok(KnowledgeGraphData {
        nodes: nodes.into_values().collect(),
        links,
    })
}

struct ChannelMeta {
    id: &'static str,
    label: &'static str,
}

const KNOWN_CHANNELS: &[ChannelMeta] = &[
    ChannelMeta {
        id: "web",
        label: "Web 控制台",
    },
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
