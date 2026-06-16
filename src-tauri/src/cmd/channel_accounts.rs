//! Channel saved accounts IPC (channel-agnostic facade).

use tauri::State;

#[cfg(feature = "channel-wework")]
use crate::context::channel::wework_accounts::WeworkAccountsStore;
use crate::services::channel::{ChannelAccountConfigDto, ChannelSavedAccountDto};

#[cfg(feature = "channel-wework")]
fn to_channel_account(
    account: crate::context::channel::wework_accounts::WeworkSavedAccountDto,
) -> ChannelSavedAccountDto {
    ChannelSavedAccountDto {
        id: account.id,
        channel: "wework".into(),
        label: account.label,
        config: ChannelAccountConfigDto {
            settings: Some(serde_json::to_value(account.config).unwrap_or_default()),
        },
        created_at: account.created_at as i32,
        last_connected_at: account.last_connected_at.map(|v| v as i32),
        external_user_id: account.wework_user_id,
        contacts_synced: account.contacts_synced,
        contacts_synced_at: account.contacts_synced_at.map(|v| v as i32),
    }
}

#[cfg(feature = "channel-wework")]
fn require_wework(channel: &str) -> Result<(), String> {
    if channel == "wework" {
        Ok(())
    } else {
        Err(format!(
            "{}: account store not implemented for {channel}",
            crate::services::channel::error_code::UNKNOWN_CHANNEL
        ))
    }
}

#[tauri::command]
pub fn channel_list_accounts(
    channel: String,
    #[cfg(feature = "channel-wework")] store: State<'_, WeworkAccountsStore>,
) -> Result<Vec<ChannelSavedAccountDto>, String> {
    #[cfg(feature = "channel-wework")]
    {
        require_wework(&channel)?;
        let accounts = store.list_accounts()?;
        Ok(accounts.into_iter().map(to_channel_account).collect())
    }
    #[cfg(not(feature = "channel-wework"))]
    {
        let _ = channel;
        Err("channel accounts not available in this build".into())
    }
}

#[tauri::command]
pub fn channel_get_active_account_id(
    channel: String,
    #[cfg(feature = "channel-wework")] store: State<'_, WeworkAccountsStore>,
) -> Result<Option<String>, String> {
    #[cfg(feature = "channel-wework")]
    {
        require_wework(&channel)?;
        store.get_active_account_id()
    }
    #[cfg(not(feature = "channel-wework"))]
    {
        let _ = channel;
        Err("channel accounts not available in this build".into())
    }
}

#[tauri::command]
pub fn channel_set_active_account_id(
    channel: String,
    id: Option<String>,
    #[cfg(feature = "channel-wework")] store: State<'_, WeworkAccountsStore>,
) -> Result<(), String> {
    #[cfg(feature = "channel-wework")]
    {
        require_wework(&channel)?;
        store.set_active_account_id(id.as_deref())
    }
    #[cfg(not(feature = "channel-wework"))]
    {
        let _ = (channel, id);
        Err("channel accounts not available in this build".into())
    }
}
