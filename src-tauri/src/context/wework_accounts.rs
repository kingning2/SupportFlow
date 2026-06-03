//! SQLite persistence for WeCom saved accounts (wework app).

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeworkAccountConfigDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wework_exe_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wework_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wework_smart: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wework_init_wait_seconds: Option<i64>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeworkSavedAccountDto {
    pub id: String,
    pub label: String,
    pub config: WeworkAccountConfigDto,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_connected_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wework_user_id: Option<String>,
}

pub struct WeworkAccountsStore {
    conn: Mutex<Connection>,
}

impl WeworkAccountsStore {
    pub fn open(app: &AppHandle) -> Result<Self, String> {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        crate::utils::fs::create_dir_all(&dir)?;
        let path = dir.join("wework_accounts.db");
        Self::open_path(path)
    }

    fn open_path(path: PathBuf) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS wework_accounts (
                id TEXT PRIMARY KEY NOT NULL,
                label TEXT NOT NULL,
                wework_user_id TEXT,
                wework_exe_path TEXT,
                wework_version TEXT,
                wework_smart INTEGER,
                wework_init_wait_seconds INTEGER,
                created_at INTEGER NOT NULL,
                last_connected_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_wework_accounts_user_id
                ON wework_accounts(wework_user_id);
            CREATE TABLE IF NOT EXISTS wework_active_account (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                account_id TEXT
            );
            "#,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn list_accounts(&self) -> Result<Vec<WeworkSavedAccountDto>, String> {
        let conn = crate::utils::err::lock_mutex(&self.conn)?;
        let mut stmt = conn
            .prepare(
                r#"SELECT id, label, wework_user_id, wework_exe_path, wework_version,
                          wework_smart, wework_init_wait_seconds, created_at, last_connected_at
                   FROM wework_accounts
                   ORDER BY COALESCE(last_connected_at, created_at) DESC"#,
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(WeworkSavedAccountDto {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    wework_user_id: row.get(2)?,
                    config: WeworkAccountConfigDto {
                        wework_exe_path: row.get(3)?,
                        wework_version: row.get(4)?,
                        wework_smart: row.get::<_, Option<i64>>(5)?.map(|v| v != 0),
                        wework_init_wait_seconds: row.get(6)?,
                    },
                    created_at: row.get(7)?,
                    last_connected_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn upsert_account(
        &self,
        account: WeworkSavedAccountDto,
    ) -> Result<WeworkSavedAccountDto, String> {
        let conn = crate::utils::err::lock_mutex(&self.conn)?;
        let smart = account
            .config
            .wework_smart
            .map(|b| if b { 1i64 } else { 0i64 });
        conn.execute(
            r#"INSERT INTO wework_accounts (
                    id, label, wework_user_id, wework_exe_path, wework_version,
                    wework_smart, wework_init_wait_seconds, created_at, last_connected_at
               ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
               ON CONFLICT(id) DO UPDATE SET
                    label = excluded.label,
                    wework_user_id = COALESCE(excluded.wework_user_id, wework_user_id),
                    wework_exe_path = excluded.wework_exe_path,
                    wework_version = excluded.wework_version,
                    wework_smart = excluded.wework_smart,
                    wework_init_wait_seconds = excluded.wework_init_wait_seconds,
                    last_connected_at = excluded.last_connected_at"#,
            params![
                account.id,
                account.label,
                account.wework_user_id,
                account.config.wework_exe_path,
                account.config.wework_version,
                smart,
                account.config.wework_init_wait_seconds,
                account.created_at,
                account.last_connected_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(account)
    }

    pub fn delete_account(&self, id: &str) -> Result<(), String> {
        let conn = crate::utils::err::lock_mutex(&self.conn)?;
        conn.execute("DELETE FROM wework_accounts WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM wework_active_account WHERE account_id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_active_account_id(&self) -> Result<Option<String>, String> {
        let conn = crate::utils::err::lock_mutex(&self.conn)?;
        let mut stmt = conn
            .prepare("SELECT account_id FROM wework_active_account WHERE id = 1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let id: Option<String> = row.get(0).map_err(|e| e.to_string())?;
            return Ok(id);
        }
        Ok(None)
    }

    pub fn set_active_account_id(&self, id: Option<&str>) -> Result<(), String> {
        let conn = crate::utils::err::lock_mutex(&self.conn)?;
        match id {
            Some(account_id) => {
                conn.execute(
                    r#"INSERT INTO wework_active_account (id, account_id) VALUES (1, ?1)
                       ON CONFLICT(id) DO UPDATE SET account_id = excluded.account_id"#,
                    params![account_id],
                )
                .map_err(|e| e.to_string())?;
            }
            None => {
                conn.execute("DELETE FROM wework_active_account WHERE id = 1", [])
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
