//! Background MCP loader (`tool_manager._load_mcp_tools`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use tracing::{info, warn};

use super::client::McpClient;
use super::config::{load_mcp_configs, mcp_json_signature, McpServerConfig};
use super::mcp_tool::McpTool;
use super::registry::{McpToolMap, McpToolRegistry};
use crate::tools::AgentTool;

/// Server load status for UI / debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpServerStatus {
    Pending,
    Ready,
    Failed,
}

impl McpServerStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

/// Loads MCP servers in the background and publishes tools into [`McpToolRegistry`].
pub struct McpToolLoader {
    workspace_dir: PathBuf,
    pub registry: Arc<McpToolRegistry>,
    clients: RwLock<HashMap<String, Arc<McpClient>>>,
    status: RwLock<HashMap<String, McpServerStatus>>,
    signature: RwLock<(Option<u64>, Option<String>)>,
    active_configs: RwLock<HashMap<String, McpServerConfig>>,
    load_started: AtomicBool,
}

impl McpToolLoader {
    pub fn new(workspace_dir: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            workspace_dir,
            registry: Arc::new(McpToolRegistry::new()),
            clients: RwLock::new(HashMap::new()),
            status: RwLock::new(HashMap::new()),
            signature: RwLock::new((None, None)),
            active_configs: RwLock::new(HashMap::new()),
            load_started: AtomicBool::new(false),
        })
    }

    /// Idempotent background load (mirrors `_load_mcp_tools`).
    pub fn ensure_background_load(self: &Arc<Self>) {
        if self.load_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let configs = load_mcp_configs(&self.workspace_dir);
        *self.signature.write().expect("signature") = mcp_json_signature(&self.workspace_dir);
        *self.active_configs.write().expect("active_configs") = configs
            .iter()
            .map(|c| (c.name.clone(), c.clone()))
            .collect();

        if configs.is_empty() {
            return;
        }

        for cfg in &configs {
            self.status
                .write()
                .expect("status")
                .insert(cfg.name.clone(), McpServerStatus::Pending);
        }

        info!(count = configs.len(), "MCP loading started in background");

        let loader = self.clone();
        tokio::spawn(async move {
            loader.load_servers(configs).await;
        });
    }

    /// Cheap mtime+sha256 check; reload added/removed/changed servers.
    pub fn refresh_if_changed(self: &Arc<Self>) {
        let new_sig = mcp_json_signature(&self.workspace_dir);
        if new_sig == *self.signature.read().expect("signature") {
            return;
        }

        let new_configs = load_mcp_configs(&self.workspace_dir);
        let new_by_name: HashMap<String, McpServerConfig> = new_configs
            .iter()
            .map(|c| (c.name.clone(), c.clone()))
            .collect();
        let old_by_name = self.active_configs.read().expect("active_configs").clone();

        let added: Vec<_> = new_by_name
            .keys()
            .filter(|n| !old_by_name.contains_key(*n))
            .cloned()
            .collect();
        let removed: Vec<_> = old_by_name
            .keys()
            .filter(|n| !new_by_name.contains_key(*n))
            .cloned()
            .collect();
        let changed: Vec<_> = new_by_name
            .keys()
            .filter(|n| {
                old_by_name
                    .get(*n)
                    .is_some_and(|old| old != new_by_name.get(*n).unwrap())
            })
            .cloned()
            .collect();

        if added.is_empty() && removed.is_empty() && changed.is_empty() {
            *self.signature.write().expect("signature") = new_sig;
            return;
        }

        info!(?added, ?removed, ?changed, "mcp.json changed — reloading");

        for name in removed.iter().chain(changed.iter()) {
            self.teardown_server(name);
        }

        let to_start: Vec<McpServerConfig> = added
            .iter()
            .chain(changed.iter())
            .filter_map(|n| new_by_name.get(n).cloned())
            .collect();

        for cfg in &to_start {
            self.status
                .write()
                .expect("status")
                .insert(cfg.name.clone(), McpServerStatus::Pending);
        }

        *self.active_configs.write().expect("active_configs") = new_by_name;
        *self.signature.write().expect("signature") = new_sig;

        if !to_start.is_empty() {
            let loader = self.clone();
            tokio::spawn(async move {
                loader.load_servers(to_start).await;
            });
        }
    }

    pub fn list_mcp_status(&self) -> HashMap<String, String> {
        self.status
            .read()
            .expect("status")
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().to_string()))
            .collect()
    }

    pub fn client(&self, server_name: &str) -> Option<Arc<McpClient>> {
        self.clients
            .read()
            .expect("clients")
            .get(server_name)
            .cloned()
    }

    fn teardown_server(&self, server_name: &str) {
        let client = self.clients.write().expect("clients").remove(server_name);
        if let Some(client) = client {
            let _name = server_name.to_string();
            tokio::spawn(async move {
                client.shutdown().await;
            });
        }
        self.registry.remove_server(server_name);
        self.status.write().expect("status").remove(server_name);
    }

    async fn load_servers(self: Arc<Self>, configs: Vec<McpServerConfig>) {
        for cfg in configs {
            let server_name = cfg.name.clone();
            match McpClient::initialize(&cfg).await {
                Ok(client) => {
                    let client = Arc::new(client);
                    let schemas = client.list_tools().await;
                    let mut added = Vec::new();
                    let mut tools_map: McpToolMap = HashMap::new();

                    for schema in schemas {
                        if schema.name.is_empty() {
                            continue;
                        }
                        let tool = Arc::new(McpTool::new(
                            client.clone(),
                            schema.clone(),
                            server_name.clone(),
                        )) as Arc<dyn AgentTool>;
                        added.push(schema.name.clone());
                        tools_map.insert(schema.name, tool);
                    }

                    self.registry.merge_tools(tools_map);
                    self.clients
                        .write()
                        .expect("clients")
                        .insert(server_name.clone(), client);
                    self.status
                        .write()
                        .expect("status")
                        .insert(server_name.clone(), McpServerStatus::Ready);
                    info!(
                        server = %server_name,
                        tools = ?added,
                        "MCP server ready"
                    );
                }
                Err(e) => {
                    self.status
                        .write()
                        .expect("status")
                        .insert(server_name.clone(), McpServerStatus::Failed);
                    warn!(server = %server_name, %e, "MCP server failed to initialize");
                }
            }
        }

        let ready = self
            .status
            .read()
            .expect("status")
            .values()
            .filter(|s| **s == McpServerStatus::Ready)
            .count();
        let total = self.status.read().expect("status").len();
        let tool_count = self.registry.tool_count();
        info!(ready, total, tool_count, "MCP loading pass complete");
    }
}
