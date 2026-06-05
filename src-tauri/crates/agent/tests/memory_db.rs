use std::sync::Arc;

use agent::memory::{DbMemoryManager, MemoryChunk, MemoryConfig, MemoryStorage};
use agent::{create_memory_manager, AgentTool, MemoryManager, MemorySearchTool};
use models::ModelsConfig;
use serde_json::json;
use tempfile::TempDir;

#[test]
fn storage_keyword_search_roundtrip() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("index.db");
    let storage = MemoryStorage::open(&db).expect("open db");
    storage
        .save_chunks_batch(&[MemoryChunk {
            id: "abc".into(),
            user_id: None,
            scope: "shared".into(),
            source: "memory".into(),
            path: "memory/notes.md".into(),
            start_line: 1,
            end_line: 1,
            text: "user prefers dark mode".into(),
            embedding: None,
            hash: MemoryStorage::compute_hash("user prefers dark mode"),
        }])
        .expect("save");
    let hits = storage
        .search_keyword("dark mode", None, &["shared"], 10)
        .expect("search");
    assert!(!hits.is_empty(), "keyword search should return hits");
}

#[tokio::test]
async fn sqlite_memory_sync_and_search() {
    let dir = TempDir::new().expect("tempdir");
    let ws = dir.path().to_path_buf();
    std::fs::write(ws.join("MEMORY.md"), "user prefers dark mode\n").expect("write");

    let cfg = MemoryConfig::new(&ws);
    let storage = Arc::new(MemoryStorage::open(&cfg.db_path()).expect("open db"));
    let manager = DbMemoryManager::new(cfg, storage.clone(), None);
    manager.sync_index().await.expect("sync");

    let hits = storage
        .search_keyword("dark mode", None, &["shared"], 10)
        .expect("search");
    assert!(!hits.is_empty(), "storage should have indexed chunks");

    let direct_hits = MemoryManager::search(&manager, "dark mode", None, 10, 0.1)
        .await
        .expect("direct manager search");
    assert!(!direct_hits.is_empty(), "direct search should return hits");

    let memory: Arc<dyn agent::MemoryManager> = Arc::new(manager);
    let search = MemorySearchTool::new(memory, None, false);
    let result = search.execute(json!({ "query": "dark mode" })).await;
    assert_eq!(result.status, "success");
    let text = result.result.as_str().unwrap_or("");
    assert!(text.contains("Found"), "tool search: {text}");
}

#[tokio::test]
async fn factory_memory_manager_indexes_workspace() {
    let dir = TempDir::new().expect("tempdir");
    let workspace = dir.path().to_path_buf();
    std::fs::write(workspace.join("MEMORY.md"), "dark mode preference\n").unwrap();

    let memory = create_memory_manager(workspace, &ModelsConfig::default(), false).expect("memory");
    memory.sync().await.expect("initial sync");
    let search = MemorySearchTool::new(memory, None, false);
    let result = search.execute(json!({ "query": "dark mode" })).await;
    assert_eq!(result.status, "success");
    let text = result.result.as_str().unwrap_or("");
    assert!(
        text.contains("Found"),
        "factory manager should index: {text}"
    );
}
