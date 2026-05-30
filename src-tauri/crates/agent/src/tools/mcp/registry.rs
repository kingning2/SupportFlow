//! MCP tool registry + `sync_mcp_into_agent` (`tool_manager.py`).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use tracing::debug;

use crate::tools::AgentTool;

pub type McpToolMap = HashMap<String, Arc<dyn AgentTool>>;

/// Registry of MCP tools loaded at runtime.
#[derive(Default)]
pub struct McpToolRegistry {
    tools: RwLock<McpToolMap>,
}

impl McpToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_tools(&self, tools: McpToolMap) {
        *self.tools.write().expect("mcp tools") = tools;
    }

    pub fn register(&self, tool: Arc<dyn AgentTool>) {
        let name = tool.name().to_string();
        self.tools.write().expect("mcp tools").insert(name, tool);
    }

    pub fn list_names(&self) -> Vec<String> {
        self.tools
            .read()
            .expect("mcp tools")
            .keys()
            .cloned()
            .collect()
    }

    pub fn tool_count(&self) -> usize {
        self.tools.read().expect("mcp tools").len()
    }

    pub fn merge_tools(&self, tools: McpToolMap) {
        let mut guard = self.tools.write().expect("mcp tools");
        for (name, tool) in tools {
            guard.insert(name, tool);
        }
    }

    /// Remove all tools belonging to an MCP server.
    pub fn remove_server(&self, server_name: &str) {
        let mut guard = self.tools.write().expect("mcp tools");
        guard.retain(|_, tool| tool.mcp_server_name() != Some(server_name));
    }

    pub fn snapshot(&self) -> McpToolMap {
        self.tools.read().expect("mcp tools").clone()
    }

    /// Reconcile agent tool map with MCP registry (built-in tools untouched).
    pub fn sync_into(&self, agent_tools: &mut HashMap<String, Arc<dyn AgentTool>>) {
        let current = self.tools.read().expect("mcp tools");
        let registry_names: HashSet<String> = current.keys().cloned().collect();

        let agent_mcp_names: HashSet<String> = agent_tools
            .iter()
            .filter(|(_, t)| t.is_mcp())
            .map(|(n, _)| n.clone())
            .collect();

        let added: Vec<_> = registry_names
            .difference(&agent_mcp_names)
            .cloned()
            .collect();
        let removed: Vec<_> = agent_mcp_names
            .difference(&registry_names)
            .cloned()
            .collect();

        if added.is_empty() && removed.is_empty() {
            return;
        }

        for name in &removed {
            agent_tools.remove(name);
        }
        for name in &added {
            if let Some(tool) = current.get(name) {
                agent_tools.insert(name.clone(), tool.clone());
            }
        }

        debug!(?added, ?removed, "MCP tools synced into agent");
    }

    /// Reconcile `Vec<Arc<dyn AgentTool>>` representation.
    pub fn sync_into_vec(&self, agent_tools: &mut Vec<Arc<dyn AgentTool>>) {
        let mut map: HashMap<String, Arc<dyn AgentTool>> = agent_tools
            .iter()
            .map(|t| (t.name().to_string(), t.clone()))
            .collect();
        self.sync_into(&mut map);
        *agent_tools = map.into_values().collect();
    }
}
