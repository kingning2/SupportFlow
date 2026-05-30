//! Memory backend trait (full `MemoryManager` port deferred; file keyword search provided).

use std::path::Path;

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct MemorySearchHit {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub score: f64,
    pub snippet: String,
}

/// Subset of Python `agent.memory.manager.MemoryManager` used by tools.
#[async_trait]
pub trait MemoryManager: Send + Sync {
    fn workspace(&self) -> &Path;

    async fn search(
        &self,
        query: &str,
        user_id: Option<&str>,
        max_results: usize,
        min_score: f64,
    ) -> Result<Vec<MemorySearchHit>, String>;

    fn mark_dirty(&self) {
        // optional hook for write/edit on memory paths
    }
}
