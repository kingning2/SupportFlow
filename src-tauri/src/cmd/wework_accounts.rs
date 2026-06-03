//! WeCom saved accounts IPC (SQLite).

use tauri::State;

use crate::context::license_store::LicenseStore;
use crate::context::wework_accounts::{WeworkAccountsStore, WeworkSavedAccountDto};

#[tauri::command]
pub fn wework_list_accounts(
    license: State<'_, LicenseStore>,
    store: State<'_, WeworkAccountsStore>,
) -> Result<Vec<WeworkSavedAccountDto>, String> {
    license.require_valid()?;
    store.list_accounts()
}

#[tauri::command]
pub fn wework_upsert_account(
    license: State<'_, LicenseStore>,
    store: State<'_, WeworkAccountsStore>,
    account: WeworkSavedAccountDto,
) -> Result<WeworkSavedAccountDto, String> {
    license.require_valid()?;
    store.upsert_account(account)
}

#[tauri::command]
pub fn wework_delete_account(
    license: State<'_, LicenseStore>,
    store: State<'_, WeworkAccountsStore>,
    id: String,
) -> Result<(), String> {
    license.require_valid()?;
    store.delete_account(&id)
}

#[tauri::command]
pub fn wework_get_active_account_id(
    license: State<'_, LicenseStore>,
    store: State<'_, WeworkAccountsStore>,
) -> Result<Option<String>, String> {
    license.require_valid()?;
    store.get_active_account_id()
}

#[tauri::command]
pub fn wework_set_active_account_id(
    license: State<'_, LicenseStore>,
    store: State<'_, WeworkAccountsStore>,
    id: Option<String>,
) -> Result<(), String> {
    license.require_valid()?;
    store.set_active_account_id(id.as_deref())
}
