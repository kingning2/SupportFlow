//! Generic channel account DTOs (channel-agnostic IPC contract).

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelAccountConfigDto {
    /// Channel-specific settings (e.g. wework_exe_path).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSavedAccountDto {
    pub id: String,
    pub channel: String,
    pub label: String,
    pub config: ChannelAccountConfigDto,
    pub created_at: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_connected_at: Option<i32>,
    /// External user id on the channel (wework user_id, telegram chat id, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_user_id: Option<String>,
    #[serde(default)]
    pub contacts_synced: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contacts_synced_at: Option<i32>,
}
