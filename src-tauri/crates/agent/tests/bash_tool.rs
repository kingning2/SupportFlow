use agent::{AgentTool, BashConfig, BashTool};
use serde_json::json;

#[tokio::test]
async fn bash_echo_success() {
    let tool = BashTool::new(BashConfig::default());
    let result = tool
        .execute(json!({ "command": if cfg!(windows) { "echo hello" } else { "echo hello" } }))
        .await;
    assert_eq!(result.status, "success");
    let output = result
        .result
        .get("output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(output.contains("hello"), "output was: {output}");
}

#[tokio::test]
async fn bash_blocks_supportflow_env() {
    let tool = BashTool::new(BashConfig::default());
    let result = tool
        .execute(json!({ "command": "cat ~/.supportflow/.env" }))
        .await;
    assert_eq!(result.status, "error");
}
