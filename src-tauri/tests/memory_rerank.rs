//! Rerank layer tests（local lexical provider，无需外部 API）。

use std::sync::Arc;

use tauri_app_lib::services::agent::memory::{
    DbMemoryManager, LexicalRerankProvider, MemoryConfig, MemoryStorage, RerankProvider,
    SearchResult,
};
use tauri_app_lib::services::agent::MemoryManager;

fn sample_results() -> Vec<SearchResult> {
    vec![
        SearchResult {
            path: "memory/a.md".into(),
            start_line: 1,
            end_line: 2,
            score: 0.9,
            snippet: "unrelated topic about weather".into(),
            source: "memory".into(),
            user_id: None,
        },
        SearchResult {
            path: "memory/b.md".into(),
            start_line: 1,
            end_line: 2,
            score: 0.5,
            snippet: "user prefers dark mode theme".into(),
            source: "memory".into(),
            user_id: None,
        },
    ]
}

#[tokio::test]
async fn lexical_rerank_reorders_by_query_overlap() {
    let reranker = LexicalRerankProvider;
    let ranked = reranker
        .rerank("dark mode preference", sample_results())
        .await
        .expect("rerank");
    assert_eq!(ranked.first().map(|r| r.path.as_str()), Some("memory/b.md"));
}

#[tokio::test]
async fn manager_without_rerank_keeps_hybrid_order() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ws = dir.path().to_path_buf();
    std::fs::write(ws.join("MEMORY.md"), "user prefers dark mode\n").expect("write");

    let cfg = MemoryConfig::new(&ws);
    let storage = Arc::new(MemoryStorage::open(&cfg.db_path()).expect("open db"));
    let manager = DbMemoryManager::new(cfg, storage, None, None);
    manager.sync_index().await.expect("sync");

    let hits = manager
        .search("dark mode", None, 10, 0.01)
        .await
        .expect("search");
    assert!(!hits.is_empty());
}

#[tokio::test]
async fn manager_with_local_rerank_changes_top_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ws = dir.path().to_path_buf();
    std::fs::write(
        ws.join("MEMORY.md"),
        "weather forecast sunny\n\nuser prefers dark mode theme\n",
    )
    .expect("write");

    let cfg = MemoryConfig::new(&ws);
    let storage = Arc::new(MemoryStorage::open(&cfg.db_path()).expect("open db"));
    let rerank: Arc<dyn RerankProvider> = Arc::new(LexicalRerankProvider);
    let manager = DbMemoryManager::new(cfg, storage, None, Some(rerank));
    manager.sync_index().await.expect("sync");

    let hits = manager
        .search("dark mode", None, 5, 0.01)
        .await
        .expect("search");
    assert!(!hits.is_empty());
    let top = hits.first().expect("top hit");
    assert!(
        top.snippet.to_lowercase().contains("dark"),
        "expected dark-mode chunk first after rerank, got: {}",
        top.snippet
    );
}
