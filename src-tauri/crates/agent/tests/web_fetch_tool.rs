use agent::{AgentTool, WebFetchTool};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn web_fetch_requires_url() {
    let dir = TempDir::new().expect("tempdir");
    let tool = WebFetchTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({})).await;
    assert_eq!(result.status, "error");
}

#[tokio::test]
async fn web_fetch_rejects_invalid_scheme() {
    let dir = TempDir::new().expect("tempdir");
    let tool = WebFetchTool::new(dir.path().to_path_buf());
    let result = tool
        .execute(json!({ "url": "ftp://example.com/file.txt" }))
        .await;
    assert_eq!(result.status, "error");
}
