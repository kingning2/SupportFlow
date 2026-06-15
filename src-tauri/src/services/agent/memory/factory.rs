//! Factory for workspace memory backends.

use std::sync::Arc;

use crate::config::ModelsConfig;

use super::config::MemoryConfig;
use super::embedding::create_embedding_provider;
use super::manager::DbMemoryManager;
use super::storage::MemoryStorage;
use crate::services::agent::tools::memory::FileKeywordMemoryManager;
use crate::services::agent::MemoryManager;

/// Create the default memory backend for a workspace.
pub fn create_memory_manager(
    workspace: std::path::PathBuf,
    models_config: &ModelsConfig,
    enable_knowledge: bool,
) -> Result<Arc<dyn MemoryManager>, String> {
    let mut mem_config = MemoryConfig::new(workspace.clone());
    mem_config.enable_knowledge = enable_knowledge;
    mem_config.embedding_provider = models_config.embedding_provider.clone();
    mem_config.embedding_model = models_config.embedding_model.clone();
    mem_config.embedding_dimensions = models_config.embedding_dimensions;

    let db_path = mem_config.db_path();
    let storage = match MemoryStorage::open(&db_path) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            tracing::warn!(
                "[MemoryManager] SQLite memory unavailable ({e}); using file keyword fallback"
            );
            return Ok(Arc::new(FileKeywordMemoryManager::new(
                workspace,
                enable_knowledge,
            )));
        }
    };

    let embedding = create_embedding_provider(models_config)?;
    if embedding.is_none() {
        tracing::info!(
            "[MemoryManager] No embedding provider; memory will use keyword search only"
        );
    }

    let manager = DbMemoryManager::new(mem_config, storage, embedding);
    manager.set_dirty();
    Ok(Arc::new(manager))
}
