use agent::{noop_uploader, FileKeywordMemoryManager};
use agent::{AgentTool, MemoryGetTool, MemorySearchTool, SendTool, WorkspaceToolConfig, WriteTool};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

fn workspace() -> (TempDir, WorkspaceToolConfig, Arc<FileKeywordMemoryManager>) {
    let dir = TempDir::new().expect("tempdir");
    let cwd = dir.path().to_path_buf();
    let ws = WorkspaceToolConfig { cwd: cwd.clone() };
    let memory = Arc::new(FileKeywordMemoryManager::new(cwd, false));
    (dir, ws, memory)
}

#[tokio::test]
async fn send_returns_file_to_send() {
    let (_dir, ws, _) = workspace();
    let write = WriteTool::new(ws.clone());
    write
        .execute(json!({ "path": "note.txt", "content": "hi" }))
        .await;

    let send = SendTool::new(ws, noop_uploader());
    let result = send.execute(json!({ "path": "note.txt" })).await;
    assert_eq!(result.status, "success");
    assert_eq!(
        result.result.get("type").and_then(|v| v.as_str()),
        Some("file_to_send")
    );
}

#[tokio::test]
async fn memory_get_reads_memory_file() {
    let (_dir, ws, memory) = workspace();
    std::fs::create_dir_all(ws.cwd.join("memory")).unwrap();
    std::fs::write(ws.cwd.join("memory/2026-05-29.md"), "line1\nline2\n").unwrap();

    let get = MemoryGetTool::new(memory, false);
    let result = get.execute(json!({ "path": "memory/2026-05-29.md" })).await;
    assert_eq!(result.status, "success");
    let text = result.result.as_str().unwrap_or("");
    assert!(text.contains("line1"));
}

#[tokio::test]
async fn memory_search_finds_keyword() {
    let (_dir, ws, memory) = workspace();
    std::fs::create_dir_all(ws.cwd.join("memory")).unwrap();
    std::fs::write(ws.cwd.join("memory/notes.md"), "user prefers dark mode\n").unwrap();

    let search = MemorySearchTool::new(memory, None, false);
    let result = search.execute(json!({ "query": "dark mode" })).await;
    assert_eq!(result.status, "success");
    let text = result.result.as_str().unwrap_or("");
    assert!(text.contains("Found"));
    assert!(text.contains("dark mode"));
}
