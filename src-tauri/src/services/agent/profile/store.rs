//! SQLite-backed user profile store (minimal KV traits per channel user).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use rusqlite::{params, Connection};
use serde_json::{Map, Value};
use tracing::debug;

const DDL: &str = r"
CREATE TABLE IF NOT EXISTS user_profiles (
    user_id    TEXT NOT NULL,
    channel    TEXT NOT NULL,
    traits     TEXT NOT NULL DEFAULT '{}',
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, channel)
);
";

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn profile_db_path(workspace: &Path) -> PathBuf {
    workspace.join("profiles").join("index.db")
}

pub struct ProfileStore {
    conn: Mutex<Connection>,
}

impl ProfileStore {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| e.to_string())?;
        conn.execute_batch(DDL).map_err(|e| e.to_string())?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn for_workspace(workspace: &Path) -> Result<Arc<Self>, String> {
        profile_store_for_path(&profile_db_path(workspace))
    }

    pub fn get_traits(&self, user_id: &str, channel: &str) -> Result<Map<String, Value>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let raw: Option<String> = conn
            .query_row(
                "SELECT traits FROM user_profiles WHERE user_id = ?1 AND channel = ?2",
                params![user_id, channel],
                |r| r.get(0),
            )
            .ok();
        let Some(json) = raw else {
            return Ok(Map::new());
        };
        match serde_json::from_str::<Value>(&json) {
            Ok(Value::Object(m)) => Ok(m),
            _ => Ok(Map::new()),
        }
    }

    pub fn update_traits(
        &self,
        user_id: &str,
        channel: &str,
        patch: Map<String, Value>,
    ) -> Result<Map<String, Value>, String> {
        let mut merged = self.get_traits(user_id, channel)?;
        for (k, v) in patch {
            if v.is_null() {
                merged.remove(&k);
            } else {
                merged.insert(k, v);
            }
        }
        let now = now_ts();
        let json =
            serde_json::to_string(&Value::Object(merged.clone())).map_err(|e| e.to_string())?;
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO user_profiles (user_id, channel, traits, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(user_id, channel) DO UPDATE SET
                traits = excluded.traits,
                updated_at = excluded.updated_at",
            params![user_id, channel, json, now],
        )
        .map_err(|e| e.to_string())?;
        Ok(merged)
    }
}

static STORE_CACHE: OnceLock<Mutex<std::collections::HashMap<PathBuf, Arc<ProfileStore>>>> =
    OnceLock::new();

pub fn profile_store_for_path(db_path: &Path) -> Result<Arc<ProfileStore>, String> {
    let cache = STORE_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    let mut guard = cache.lock().map_err(|e| e.to_string())?;
    if let Some(s) = guard.get(db_path) {
        return Ok(s.clone());
    }
    let store = Arc::new(ProfileStore::open(db_path)?);
    debug!("[ProfileStore] opened {}", db_path.display());
    guard.insert(db_path.to_path_buf(), store.clone());
    Ok(store)
}
