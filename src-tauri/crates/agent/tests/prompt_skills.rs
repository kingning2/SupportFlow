//! Prompt builder + skills + MCP registry smoke tests.

use std::sync::Arc;

use agent::{
    build_agent_system_prompt, load_context_files, AgentTool, McpToolRegistry, SkillManager,
};
use async_trait::async_trait;
use serde_json::Value;
use tempfile::TempDir;

struct StubTool {
    name: &'static str,
}

#[async_trait]
impl AgentTool for StubTool {
    fn name(&self) -> &str {
        self.name
    }
    fn description(&self) -> &str {
        "stub"
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({"type": "object"})
    }
    async fn execute(&self, _params: Value) -> agent::ToolRunResult {
        agent::ToolRunResult::success_text("ok")
    }
}

#[test]
fn load_context_files_skips_templates() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("AGENT.md"), "Hello agent").unwrap();
    std::fs::write(dir.path().join("USER.md"), "<!-- template -->").unwrap();

    let files = load_context_files(dir.path(), None);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "AGENT.md");
}

#[test]
fn build_system_prompt_includes_tooling() {
    let dir = TempDir::new().unwrap();
    let tools: Vec<Arc<dyn AgentTool>> = vec![Arc::new(StubTool { name: "read" })];
    let prompt = build_agent_system_prompt(dir.path(), &tools, None, None, false, false, None);
    assert!(prompt.contains("工具系统"));
    assert!(prompt.contains("read"));
}

struct McpStub {
    name: &'static str,
    server: &'static str,
}

#[async_trait]
impl AgentTool for McpStub {
    fn name(&self) -> &str {
        self.name
    }
    fn description(&self) -> &str {
        "mcp stub"
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object"})
    }
    fn is_mcp(&self) -> bool {
        true
    }
    fn mcp_server_name(&self) -> Option<&str> {
        Some(self.server)
    }
    async fn execute(&self, _params: serde_json::Value) -> agent::ToolRunResult {
        agent::ToolRunResult::success_text("pong")
    }
}

#[test]
fn mcp_registry_syncs_into_executor_tools() {
    let registry = Arc::new(McpToolRegistry::new());
    let mcp_tool: Arc<dyn AgentTool> = Arc::new(McpStub {
        name: "mcp_ping",
        server: "srv",
    });
    let mut map = std::collections::HashMap::new();
    map.insert("mcp_ping".into(), mcp_tool);
    registry.set_tools(map);

    let builtin: Arc<dyn AgentTool> = Arc::new(StubTool { name: "read" });
    let mut agent_tools: std::collections::HashMap<String, Arc<dyn AgentTool>> =
        std::collections::HashMap::from([("read".into(), builtin)]);

    registry.sync_into(&mut agent_tools);
    assert!(agent_tools.contains_key("read"));
    assert!(agent_tools.contains_key("mcp_ping"));
    assert!(agent_tools.get("mcp_ping").unwrap().is_mcp());
}

#[test]
fn skill_manager_formats_prompt_block() {
    let dir = TempDir::new().unwrap();
    let skill_dir = dir.path().join("skills").join("demo-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: demo-skill\ndescription: Demo skill\n---\n# Demo",
    )
    .unwrap();

    let mgr = SkillManager::new(dir.path(), Some(dir.path().join("skills")));
    let prompt = mgr.build_skills_prompt(None);
    assert!(prompt.contains("available_skills") || prompt.contains("demo-skill"));
}
