//! WeCom saved accounts IPC (SQLite).

use tauri::State;

use crate::context::wework_accounts::{WeworkAccountsStore, WeworkSavedAccountDto};

#[tauri::command]
pub fn wework_list_accounts(
    store: State<'_, WeworkAccountsStore>,
) -> Result<Vec<WeworkSavedAccountDto>, String> {
    store.list_accounts()
}

#[tauri::command]
pub fn wework_upsert_account(
    store: State<'_, WeworkAccountsStore>,
    account: WeworkSavedAccountDto,
) -> Result<WeworkSavedAccountDto, String> {
    store.upsert_account(account)
}

#[tauri::command]
pub fn wework_delete_account(
    store: State<'_, WeworkAccountsStore>,
    id: String,
) -> Result<(), String> {
    store.delete_account(&id)
}

#[tauri::command]
pub fn wework_get_active_account_id(
    store: State<'_, WeworkAccountsStore>,
) -> Result<Option<String>, String> {
    store.get_active_account_id()
}

#[tauri::command]
pub fn wework_set_active_account_id(
    store: State<'_, WeworkAccountsStore>,
    id: Option<String>,
) -> Result<(), String> {
    store.set_active_account_id(id.as_deref())
}
