//! Long-term memory subsystem (`agent/memory/`).

mod chunker;
mod config;
mod conversation_restore;
mod conversation_store;
mod embedding;
mod factory;
mod manager;
mod storage;

pub use config::MemoryConfig;
pub use conversation_store::{
    conversation_store_for_workspace, persist_agent_run, restore_agent_messages, ConversationStore,
};
pub use factory::create_memory_manager;
pub use manager::DbMemoryManager;
pub use storage::{MemoryChunk, MemoryStorage};
