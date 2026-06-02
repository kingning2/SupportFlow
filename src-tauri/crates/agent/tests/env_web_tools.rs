use std::sync::Arc;

use agent::{AgentTool, EnvConfigTool, EnvConfigToolConfig, WebSearchTool};
use models::{ModelsConfig, ToolsConfig, WebSearchConfig};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn env_config_set_and_list() {
    let dir = TempDir::new().expect("tempdir");
    let env_path = dir.path().join(".env");
    let tool = EnvConfigTool::new(EnvConfigToolConfig {
        env_path: Some(env_path),
        on_change: None,
    });
    let set = tool
        .execute(json!({
            "action": "set",
            "key": "OPENAI_API_KEY",
            "value": "sk-test-1234567890"
        }))
        .await;
    assert_eq!(set.status, "success");
    let list = tool.execute(json!({ "action": "list" })).await;
    assert_eq!(list.status, "success");
    assert!(list.result["variables"]["OPENAI_API_KEY"].is_object());
}

#[test]
fn web_search_not_registered_without_keys() {
    let cfg = ModelsConfig::default();
    assert!(!WebSearchTool::is_available(&cfg));
}

#[test]
fn web_search_available_with_bocha_key() {
    let cfg = ModelsConfig {
        tools: Some(ToolsConfig {
            web_search: Some(WebSearchConfig {
                bocha_api_key: Some("test".into()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(WebSearchTool::is_available(&cfg));
}

#[test]
fn web_search_schema_includes_provider_when_multiple_keys() {
    let cfg = ModelsConfig {
        tools: Some(ToolsConfig {
            web_search: Some(WebSearchConfig {
                bocha_api_key: Some("b".into()),
                strategy: Some("auto".into()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        linkai_api_key: Some("l".into()),
        ..Default::default()
    };
    let tool = WebSearchTool::new(Arc::new(cfg));
    let schema = tool.input_schema();
    assert!(schema["properties"]["provider"].is_object());
}
