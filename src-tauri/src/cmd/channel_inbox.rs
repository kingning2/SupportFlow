//! Channel inbox IPC (SQLite + event broadcast, all channels).

use tauri::State;

use crate::context::channel::{ChannelInboxSnapshotDto, ChannelInboxStore};

#[tauri::command]
pub fn channel_get_inbox(
    store: State<'_, ChannelInboxStore>,
    channel: Option<String>,
) -> Result<ChannelInboxSnapshotDto, String> {
    let channel_ref = channel.as_deref();
    crate::log_cmd_result!(
        "cmd.channel.get_inbox",
        store.snapshot(channel_ref),
        "channel={}",
        channel_ref.unwrap_or("all")
    )
}
