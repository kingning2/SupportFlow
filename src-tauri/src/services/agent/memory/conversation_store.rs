//! SQLite conversation persistence (`agent/memory/conversation_store.py`).
//!
//! Shares the memory DB at `{workspace}/memory/long-term/index.db`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use rusqlite::{params, Connection};
use serde_json::Value;
use tracing::{debug, info, warn};

use super::config::MemoryConfig;
use super::conversation_restore::{filter_text_only_messages, strip_thinking_blocks};

const DDL: &str = r"
CREATE TABLE IF NOT EXISTS sessions (
    session_id        TEXT    PRIMARY KEY,
    channel_type      TEXT    NOT NULL DEFAULT '',
    title             TEXT    NOT NULL DEFAULT '',
    context_start_seq INTEGER NOT NULL DEFAULT 0,
    created_at        INTEGER NOT NULL,
    last_active       INTEGER NOT NULL,
    msg_count         INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS messages (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id   TEXT    NOT NULL,
    seq          INTEGER NOT NULL,
    role         TEXT    NOT NULL,
    content      TEXT    NOT NULL,
    created_at   INTEGER NOT NULL,
    extras       TEXT    NOT NULL DEFAULT '',
    UNIQUE (session_id, seq)
);
CREATE INDEX IF NOT EXISTS idx_messages_session ON messages (session_id, seq);
CREATE INDEX IF NOT EXISTS idx_sessions_last_active ON sessions (last_active);
";

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn is_visible_user_content(raw: &str) -> bool {
    let content: Value = serde_json::from_str(raw).unwrap_or(Value::String(raw.to_string()));
    match &content {
        Value::String(s) => !s.trim().is_empty(),
        Value::Array(blocks) => blocks.iter().any(|b| {
            b.get("type")
                .and_then(|t| t.as_str())
                .is_some_and(|t| t == "text")
        }),
        _ => false,
    }
}

fn extract_title_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.trim().to_string(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| {
                if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                    b.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string(),
        _ => String::new(),
    }
}

pub struct ConversationStore {
    conn: Mutex<Connection>,
}

impl ConversationStore {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| e.to_string())?;
        conn.execute_batch(DDL).map_err(|e| e.to_string())?;
        Self::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn for_workspace(workspace: &Path) -> Result<Arc<Self>, String> {
        let db_path = MemoryConfig::new(workspace).db_path();
        conversation_store_for_path(&db_path)
    }

    fn migrate(conn: &Connection) -> Result<(), String> {
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(sessions)")
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get(1))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        if !cols.iter().any(|c| c == "channel_type") {
            let _ = conn.execute(
                "ALTER TABLE sessions ADD COLUMN channel_type TEXT NOT NULL DEFAULT ''",
                [],
            );
        }
        if !cols.iter().any(|c| c == "title") {
            let _ = conn.execute(
                "ALTER TABLE sessions ADD COLUMN title TEXT NOT NULL DEFAULT ''",
                [],
            );
        }
        if !cols.iter().any(|c| c == "context_start_seq") {
            let _ = conn.execute(
                "ALTER TABLE sessions ADD COLUMN context_start_seq INTEGER NOT NULL DEFAULT 0",
                [],
            );
        }
        let msg_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(messages)")
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get(1))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        if !msg_cols.iter().any(|c| c == "extras") {
            let _ = conn.execute(
                "ALTER TABLE messages ADD COLUMN extras TEXT NOT NULL DEFAULT ''",
                [],
            );
        }
        Ok(())
    }

    pub fn load_messages(&self, session_id: &str, max_turns: u32) -> Result<Vec<Value>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let ctx_start: i64 = conn
            .query_row(
                "SELECT context_start_seq FROM sessions WHERE session_id = ?1",
                params![session_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let mut stmt = conn
            .prepare(
                "SELECT seq, role, content FROM messages
                 WHERE session_id = ?1 AND seq >= ?2
                 ORDER BY seq DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i64, String, String)> = stmt
            .query_map(params![session_id, ctx_start], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut visible_seqs = Vec::new();
        for (seq, role, raw) in &rows {
            if role == "user" && is_visible_user_content(raw) {
                visible_seqs.push(*seq);
            }
        }

        let cutoff = if visible_seqs.len() <= max_turns as usize {
            None
        } else {
            Some(visible_seqs[max_turns as usize - 1])
        };

        let mut result = Vec::new();
        for (seq, role, raw) in rows.into_iter().rev() {
            if let Some(c) = cutoff {
                if seq < c {
                    continue;
                }
            }
            let content: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            let content = if role == "assistant" {
                if let Value::Array(blocks) = content {
                    Value::Array(
                        blocks
                            .into_iter()
                            .filter(|b| b.get("type").and_then(|t| t.as_str()) != Some("thinking"))
                            .collect(),
                    )
                } else {
                    content
                }
            } else {
                content
            };
            result.push(serde_json::json!({"role": role, "content": content}));
        }
        Ok(result)
    }

    pub fn append_messages(
        &self,
        session_id: &str,
        messages: &[Value],
        channel_type: &str,
    ) -> Result<(), String> {
        if messages.is_empty() {
            return Ok(());
        }
        let now = now_ts();
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT OR IGNORE INTO sessions
                (session_id, channel_type, created_at, last_active, msg_count)
             VALUES (?1, ?2, ?3, ?3, 0)",
            params![session_id, channel_type, now],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE sessions SET last_active = ?1 WHERE session_id = ?2",
            params![now, session_id],
        )
        .map_err(|e| e.to_string())?;

        let max_seq: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(seq), -1) FROM messages WHERE session_id = ?1",
                params![session_id],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        let mut seq = max_seq + 1;

        for msg in messages {
            let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
            let content =
                serde_json::to_string(&msg.get("content").cloned().unwrap_or(Value::Null))
                    .map_err(|e| e.to_string())?;
            let extras = msg
                .get("extras")
                .map(|e| serde_json::to_string(e).unwrap_or_default())
                .unwrap_or_default();
            tx.execute(
                "INSERT OR IGNORE INTO messages
                    (session_id, seq, role, content, created_at, extras)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![session_id, seq, role, content, now, extras],
            )
            .map_err(|e| e.to_string())?;
            seq += 1;
        }

        tx.execute(
            "UPDATE sessions SET msg_count = (
                SELECT COUNT(*) FROM messages WHERE session_id = ?1
             ) WHERE session_id = ?1",
            params![session_id],
        )
        .map_err(|e| e.to_string())?;

        let title_row: Option<String> = tx
            .query_row(
                "SELECT title FROM sessions WHERE session_id = ?1",
                params![session_id],
                |r| r.get(0),
            )
            .ok();
        if title_row.as_deref().unwrap_or("").is_empty() {
            for msg in messages {
                if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                    let text = extract_title_text(msg.get("content").unwrap_or(&Value::Null));
                    if !text.is_empty() {
                        let title: String = text.chars().take(50).collect();
                        let title = title.split('\n').next().unwrap_or(&title).to_string();
                        tx.execute(
                            "UPDATE sessions SET title = ?1 WHERE session_id = ?2",
                            params![title, session_id],
                        )
                        .map_err(|e| e.to_string())?;
                        break;
                    }
                }
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_context(&self, session_id: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let max_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), -1) FROM messages WHERE session_id = ?1",
                params![session_id],
                |r| r.get(0),
            )
            .unwrap_or(-1);
        let new_start = max_seq + 1;
        conn.execute(
            "UPDATE sessions SET context_start_seq = ?1 WHERE session_id = ?2",
            params![new_start, session_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(new_start)
    }

    pub fn clear_session(&self, session_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![session_id],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM sessions WHERE session_id = ?1",
            params![session_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

static STORE_CACHE: OnceLock<Mutex<std::collections::HashMap<PathBuf, Arc<ConversationStore>>>> =
    OnceLock::new();

pub fn conversation_store_for_path(db_path: &Path) -> Result<Arc<ConversationStore>, String> {
    let cache = STORE_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    let mut guard = cache.lock().map_err(|e| e.to_string())?;
    if let Some(s) = guard.get(db_path) {
        return Ok(s.clone());
    }
    let store = Arc::new(ConversationStore::open(db_path)?);
    debug!("[ConversationStore] opened {}", db_path.display());
    guard.insert(db_path.to_path_buf(), store.clone());
    Ok(store)
}

pub fn conversation_store_for_workspace(
    workspace: &Path,
) -> Result<Arc<ConversationStore>, String> {
    ConversationStore::for_workspace(workspace)
}

/// Restore filtered history into an agent (`AgentInitializer._restore_conversation_history`).
pub fn restore_agent_messages(
    agent: &crate::services::agent::Agent,
    session_id: &str,
    workspace: &Path,
    config: &models::ModelsConfig,
) {
    if !config.conversation_persistence.unwrap_or(true) {
        return;
    }
    let max_turns = config.agent_max_context_turns.unwrap_or(20);
    let restore_turns = if session_id.starts_with("scheduler_") {
        (max_turns / 5).max(1)
    } else {
        (max_turns / 6).max(3)
    };

    let store = match conversation_store_for_workspace(workspace) {
        Ok(s) => s,
        Err(e) => {
            warn!("[ConversationStore] open failed: {e}");
            return;
        }
    };

    let saved = match store.load_messages(session_id, restore_turns) {
        Ok(m) => m,
        Err(e) => {
            warn!("[ConversationStore] load failed session={session_id}: {e}");
            return;
        }
    };
    if saved.is_empty() {
        return;
    }
    let filtered = filter_text_only_messages(&saved);
    if filtered.is_empty() {
        return;
    }
    *agent.messages.lock().expect("messages") = filtered;
    info!(
        "[AgentInitializer] Restored {} messages (from {} raw) session={}",
        agent.messages.lock().expect("messages").len(),
        saved.len(),
        session_id
    );
}

pub fn persist_agent_run(
    workspace: &Path,
    config: &models::ModelsConfig,
    session_id: &str,
    channel_type: &str,
    new_messages: &[Value],
) {
    if !config.conversation_persistence.unwrap_or(true) || new_messages.is_empty() {
        return;
    }
    let store = match conversation_store_for_workspace(workspace) {
        Ok(s) => s,
        Err(e) => {
            warn!("[AgentBridge] persist open failed: {e}");
            return;
        }
    };
    let to_store = if config.enable_thinking() {
        new_messages.to_vec()
    } else {
        strip_thinking_blocks(new_messages)
    };
    if let Err(e) = store.append_messages(session_id, &to_store, channel_type) {
        warn!("[AgentBridge] append_messages failed session={session_id}: {e}");
    }
}
