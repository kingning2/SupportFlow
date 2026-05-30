//! MCP server config load + normalize (`tool_manager._normalize_mcp_configs`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use tracing::{info, warn};

/// Normalized MCP server entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub transport: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// `mcp.json` top-level shape (Claude/Cursor style).
#[derive(Debug, Deserialize)]
struct McpJsonFile {
    #[serde(rename = "mcpServers")]
    mcp_servers: Option<Value>,
    #[serde(rename = "mcp_servers")]
    mcp_servers_snake: Option<Value>,
}

const STREAMABLE_HTTP_ALIASES: &[&str] = &[
    "streamable-http",
    "streamable_http",
    "streamablehttp",
    "http",
];

/// Normalize transport string to internal keys: `stdio` | `sse` | `streamable-http`.
pub fn normalize_transport(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    if STREAMABLE_HTTP_ALIASES.contains(&lower.as_str()) {
        "streamable-http".into()
    } else {
        lower
    }
}

/// Convert list or `mcpServers` dict to internal list format.
pub fn normalize_mcp_configs(raw: Value) -> Vec<McpServerConfig> {
    match raw {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
        Value::Object(map) => map
            .into_iter()
            .filter_map(|(name, cfg)| entry_from_dict(&name, cfg))
            .collect(),
        _ => Vec::new(),
    }
}

fn entry_from_dict(name: &str, cfg: Value) -> Option<McpServerConfig> {
    let mut obj = cfg.as_object()?.clone();
    obj.insert("name".into(), Value::String(name.to_string()));
    if !obj.contains_key("type") {
        let transport = if obj.contains_key("url") {
            "sse"
        } else {
            "stdio"
        };
        obj.insert("type".into(), Value::String(transport.into()));
    }
    serde_json::from_value(Value::Object(obj)).ok()
}

/// Load MCP configs: `{workspace}/mcp.json` first, else empty.
pub fn load_mcp_configs(workspace_dir: &Path) -> Vec<McpServerConfig> {
    let path = mcp_json_path(workspace_dir);
    if !path.is_file() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(text) => parse_mcp_json(&text, &path),
        Err(e) => {
            warn!(path = %path.display(), %e, "Failed to read mcp.json");
            Vec::new()
        }
    }
}

pub fn mcp_json_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("mcp.json")
}

fn parse_mcp_json(text: &str, path: &Path) -> Vec<McpServerConfig> {
    let data: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            warn!(path = %path.display(), %e, "Invalid mcp.json");
            return Vec::new();
        }
    };

    let raw = if let Ok(file) = serde_json::from_value::<McpJsonFile>(data.clone()) {
        file.mcp_servers.or(file.mcp_servers_snake).unwrap_or(data)
    } else {
        data.get("mcpServers")
            .or_else(|| data.get("mcp_servers"))
            .cloned()
            .unwrap_or(data)
    };

    info!(path = %path.display(), "Loading MCP config");
    normalize_mcp_configs(raw)
}

/// `(mtime_secs, sha256_hex)` for hot-reload cheap check.
pub fn mcp_json_signature(workspace_dir: &Path) -> (Option<u64>, Option<String>) {
    let path = mcp_json_path(workspace_dir);
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => return (None, None),
    };
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return (mtime, None),
    };
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    let hex = digest.iter().map(|b| format!("{b:02x}")).collect();
    (mtime, Some(hex))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalize_dict_infers_stdio() {
        let raw = json!({
            "fetch": { "command": "uvx", "args": ["mcp-server-fetch"] }
        });
        let configs = normalize_mcp_configs(raw);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].name, "fetch");
        assert_eq!(configs[0].transport, "stdio");
    }

    #[test]
    fn normalize_dict_infers_sse_from_url() {
        let raw = json!({
            "remote": { "url": "http://localhost/sse" }
        });
        let configs = normalize_mcp_configs(raw);
        assert_eq!(configs[0].transport, "sse");
    }
}
