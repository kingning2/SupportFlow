//! Channel inbox IPC (SQLite + event broadcast, all channels).

use tauri::State;

use crate::context::channel::{ChannelInboxSnapshotDto, ChannelInboxStore};
use crate::context::license_store::LicenseStore;

#[tauri::command]
pub fn channel_get_inbox(
    license: State<'_, LicenseStore>,
    store: State<'_, ChannelInboxStore>,
    channel: Option<String>,
) -> Result<ChannelInboxSnapshotDto, String> {
    license.require_valid()?;
    let channel_ref = channel.as_deref();
    crate::log_cmd_result!(
        "cmd.channel.get_inbox",
        store.snapshot(channel_ref),
        "channel={}",
        channel_ref.unwrap_or("all")
    )
}
