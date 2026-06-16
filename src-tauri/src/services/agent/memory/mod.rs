//! Long-term memory subsystem (`agent/memory/`).

mod chunker;
mod config;
mod conversation_restore;
mod conversation_store;
mod embedding;
mod eval;
mod factory;
mod manager;
mod rerank;
mod storage;

pub use config::MemoryConfig;
pub use conversation_store::{
    conversation_store_for_workspace, persist_agent_run, restore_agent_messages, ConversationStore,
};
pub use eval::{
    fixture_workspace, load_suite, print_comparison_table, run_comparison, RagEvalMetrics,
    RagEvalRun, RagEvalSuite,
};
pub use factory::create_memory_manager;
pub use manager::DbMemoryManager;
pub use rerank::{create_rerank_provider, LexicalRerankProvider, RerankProvider};
pub use storage::{MemoryChunk, MemoryStorage, SearchResult};
