//! Agent 工作区数据与服务：会话索引、知识库列表、渠道行数据。

mod data;
mod service;

pub use data::{
    build_knowledge_graph, list_channels_from_config, list_knowledge_files, list_session_summaries,
    read_knowledge_file, remove_knowledge_file, upsert_session_index, ChannelRow, KnowledgeFileRow,
    KnowledgeGraphData, KnowledgeGraphLinkRow, KnowledgeGraphNodeRow, SessionRow,
};
pub use service::AgentWorkspaceService;
