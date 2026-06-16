//! Memory configuration (`agent/memory/config.py`).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub workspace_root: PathBuf,
    pub embedding_provider: Option<String>,
    pub embedding_model: Option<String>,
    pub embedding_dimensions: Option<u32>,
    pub rerank_provider: Option<String>,
    pub rerank_model: Option<String>,
    pub chunk_max_tokens: usize,
    pub chunk_overlap_tokens: usize,
    pub max_results: usize,
    pub min_score: f64,
    pub vector_weight: f64,
    pub keyword_weight: f64,
    pub sync_on_search: bool,
    pub enable_knowledge: bool,
}

impl MemoryConfig {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            embedding_provider: None,
            embedding_model: None,
            embedding_dimensions: None,
            rerank_provider: None,
            rerank_model: None,
            chunk_max_tokens: 500,
            chunk_overlap_tokens: 50,
            max_results: 10,
            min_score: 0.05,
            vector_weight: 0.7,
            keyword_weight: 0.3,
            sync_on_search: true,
            enable_knowledge: true,
        }
    }

    pub fn workspace(&self) -> &Path {
        &self.workspace_root
    }

    pub fn memory_dir(&self) -> PathBuf {
        self.workspace_root.join("memory")
    }

    pub fn db_path(&self) -> PathBuf {
        self.memory_dir().join("long-term").join("index.db")
    }
}
