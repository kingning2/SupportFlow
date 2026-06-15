//! WeCom saved accounts IPC (SQLite).

use tauri::State;

use crate::context::channel::wework_accounts::{WeworkAccountsStore, WeworkSavedAccountDto};

#[tauri::command]
pub fn wework_list_accounts(
    store: State<'_, WeworkAccountsStore>,
) -> Result<Vec<WeworkSavedAccountDto>, String> {
    crate::log_cmd_result!("cmd.wework.list_accounts", store.list_accounts())
}

#[tauri::command]
pub fn wework_upsert_account(
    store: State<'_, WeworkAccountsStore>,
    account: WeworkSavedAccountDto,
) -> Result<WeworkSavedAccountDto, String> {
    let account_id = account.id.clone();
    crate::log_cmd_result!(
        "cmd.wework.upsert_account",
        store.upsert_account(account),
        "id={account_id}"
    )
}

#[tauri::command]
pub fn wework_delete_account(
    store: State<'_, WeworkAccountsStore>,
    id: String,
) -> Result<(), String> {
    crate::log_cmd_result!(
        "cmd.wework.delete_account",
        store.delete_account(&id),
        "id={id}"
    )
}

#[tauri::command]
pub fn wework_get_active_account_id(
    store: State<'_, WeworkAccountsStore>,
) -> Result<Option<String>, String> {
    crate::log_cmd_result!(
        "cmd.wework.get_active_account_id",
        store.get_active_account_id()
    )
}

#[tauri::command]
pub fn wework_set_active_account_id(
    store: State<'_, WeworkAccountsStore>,
    id: Option<String>,
) -> Result<(), String> {
    let active_id = id.as_deref().unwrap_or("none").to_string();
    crate::log_cmd_result!(
        "cmd.wework.set_active_account_id",
        store.set_active_account_id(id.as_deref()),
        "id={active_id}"
    )
}

#[tauri::command]
pub fn wework_mark_contacts_synced(
    store: State<'_, WeworkAccountsStore>,
    wework_user_id: String,
    synced_at: i64,
) -> Result<(), String> {
    crate::log_cmd_result!(
        "cmd.wework.mark_contacts_synced",
        store.mark_contacts_synced(&wework_user_id, synced_at),
        "wework_user_id={wework_user_id} synced_at={synced_at}"
    )
}

#[tauri::command]
pub fn wework_contacts_synced(
    store: State<'_, WeworkAccountsStore>,
    wework_user_id: String,
) -> Result<bool, String> {
    crate::log_cmd_result!(
        "cmd.wework.contacts_synced",
        store.contacts_synced(&wework_user_id),
        "wework_user_id={wework_user_id}"
    )
}
