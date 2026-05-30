use agent::AgentTool;
use agent::{EditTool, LsTool, ReadTool, WorkspaceToolConfig, WriteTool};
use serde_json::json;
use tempfile::TempDir;

fn workspace() -> (TempDir, WorkspaceToolConfig) {
    let dir = TempDir::new().expect("tempdir");
    let config = WorkspaceToolConfig {
        cwd: dir.path().to_path_buf(),
    };
    (dir, config)
}

#[tokio::test]
async fn write_read_roundtrip() {
    let (_dir, ws) = workspace();
    let write = WriteTool::new(ws.clone());
    let read = ReadTool::new(ws);

    write
        .execute(json!({ "path": "hello.txt", "content": "line1\nline2\n" }))
        .await;
    let result = read.execute(json!({ "path": "hello.txt" })).await;
    assert_eq!(result.status, "success");
    let content = result
        .result
        .get("content")
        .and_then(|c| c.as_str())
        .unwrap();
    assert!(content.contains("line1"));
    assert!(content.contains("line2"));
}

#[tokio::test]
async fn edit_replace_text() {
    let (_dir, ws) = workspace();
    let write = WriteTool::new(ws.clone());
    let edit = EditTool::new(ws.clone());
    let read = ReadTool::new(ws);

    write
        .execute(json!({ "path": "f.txt", "content": "hello world" }))
        .await;
    let edit_result = edit
        .execute(json!({
            "path": "f.txt",
            "oldText": "world",
            "newText": "rust"
        }))
        .await;
    assert_eq!(edit_result.status, "success");

    let result = read.execute(json!({ "path": "f.txt" })).await;
    let content = result
        .result
        .get("content")
        .and_then(|c| c.as_str())
        .unwrap();
    assert!(content.contains("hello rust"));
}

#[tokio::test]
async fn ls_lists_files() {
    let (_dir, ws) = workspace();
    let write = WriteTool::new(ws.clone());
    let ls = LsTool::new(ws);

    write
        .execute(json!({ "path": "a.txt", "content": "x" }))
        .await;
    let result = ls.execute(json!({ "path": "." })).await;
    assert_eq!(result.status, "success");
    let output = result
        .result
        .get("output")
        .and_then(|o| o.as_str())
        .unwrap();
    assert!(output.contains("a.txt"));
}
