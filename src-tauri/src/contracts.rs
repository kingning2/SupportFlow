//! 前后端 IPC 合约类型，由 `typeshare` 扫描生成 TypeScript。

pub use crate::context::channel::{
    ChannelConversationSummaryDto, ChannelInboxMessagePayload, ChannelInboxSnapshotDto,
    ChannelMessageDto,
};
pub use crate::events::payloads::{
    AgentConsoleState, AgentRunFinished, AgentStreamChunk, SkillItem, ToolItem,
};
pub use crate::services::channel::{ChannelAdapterCapability, ChannelTypeId};
