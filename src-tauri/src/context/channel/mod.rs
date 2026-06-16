//! 渠道 sidecar、收件箱、配置与控制台 API。

mod bridge;
mod catalog;
mod console_api;
mod inbox;
mod status;

#[cfg(feature = "channel-wework")]
pub mod wework_accounts;

#[cfg(feature = "channel-wework")]
pub use wework_accounts::WeworkAccountsStore;

pub use crate::services::channel::{
    action_response, connect_channel, disconnect_channel, persist_channel_config,
    should_restart_channel,
};
pub use bridge::ChannelBridge;
pub use catalog::{build_catalog, validate_channel_id};
pub use console_api::dispatch;
pub use inbox::{
    ChannelConversationSummaryDto, ChannelInboxMessagePayload, ChannelInboxSnapshotDto,
    ChannelInboxStore, ChannelMessageDto,
};
pub use status::ChannelStatusStore;
