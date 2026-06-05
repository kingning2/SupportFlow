//! Knowledge base (`agent/knowledge/` in SupportFlow Agent).

pub mod document_parser;
pub mod ingest;
pub mod markitdown;
pub mod service;

pub use document_parser::{
    all_doc_suffixes, is_supported_filename, is_supported_suffix, parse_document_file,
    MAX_INGEST_BYTES,
};
pub use ingest::{
    ingest_bytes, ingest_files, supported_suffixes_list, trigger_memory_sync, IngestBatchResult,
    IngestError, IngestResult, INGEST_MAX_BYTES, INGEST_MAX_LINES,
};
pub use service::{
    KnowledgeFileEntry, KnowledgeGraph, KnowledgeGraphLink, KnowledgeGraphNode,
    KnowledgeReadResult, KnowledgeService, KnowledgeTree, KnowledgeTreeNode, KnowledgeTreeStats,
};
