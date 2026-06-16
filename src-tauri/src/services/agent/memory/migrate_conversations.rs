//! One-time migration: sessions/messages from the legacy shared memory DB
//! (`memory/long-term/index.db`) into `{workspace}/conversations/index.db`.

use std::path::Path;

use rusqlite::{params, Connection};
use tracing::info;

use super::config::MemoryConfig;
use super::conversation_store::ConversationStore;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub migrated: bool,
    pub sessions_copied: usize,
    pub messages_copied: usize,
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![name],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

fn copy_sessions(old: &Connection, new: &Connection) -> Result<usize, String> {
    let mut stmt = old
        .prepare(
            "SELECT session_id, channel_type, title, context_start_seq,
                    created_at, last_active, msg_count
             FROM sessions",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, String, i64, i64, i64, i64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut copied = 0usize;
    for (session_id, channel_type, title, context_start_seq, created_at, last_active, msg_count) in
        rows
    {
        let changed = new
            .execute(
                "INSERT OR IGNORE INTO sessions
                    (session_id, channel_type, title, context_start_seq,
                     created_at, last_active, msg_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    session_id,
                    channel_type,
                    title,
                    context_start_seq,
                    created_at,
                    last_active,
                    msg_count
                ],
            )
            .map_err(|e| e.to_string())?;
        if changed > 0 {
            copied += 1;
        }
    }
    Ok(copied)
}

fn copy_messages(old: &Connection, new: &Connection) -> Result<usize, String> {
    let mut stmt = old
        .prepare(
            "SELECT session_id, seq, role, content, created_at, extras
             FROM messages",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, i64, String, String, i64, String)> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut copied = 0usize;
    for (session_id, seq, role, content, created_at, extras) in rows {
        let changed = new
            .execute(
                "INSERT OR IGNORE INTO messages
                    (session_id, seq, role, content, created_at, extras)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![session_id, seq, role, content, created_at, extras],
            )
            .map_err(|e| e.to_string())?;
        if changed > 0 {
            copied += 1;
        }
    }
    Ok(copied)
}

/// Copy legacy conversation tables into the dedicated conversation DB.
///
/// Idempotent: rows already present in the target DB are skipped.
pub fn migrate_conversations_for_workspace(workspace: &Path) -> Result<MigrationReport, String> {
    let cfg = MemoryConfig::new(workspace);
    let old_path = cfg.db_path();
    let new_path = cfg.conversation_db_path();

    if !old_path.is_file() {
        return Ok(MigrationReport::default());
    }

    let old = Connection::open(&old_path).map_err(|e| e.to_string())?;
    if !table_exists(&old, "sessions")? || !table_exists(&old, "messages")? {
        return Ok(MigrationReport::default());
    }

    let session_count: i64 = old
        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if session_count == 0 {
        return Ok(MigrationReport::default());
    }

    // Ensure target schema exists.
    let _store = ConversationStore::open(&new_path)?;
    let new = Connection::open(&new_path).map_err(|e| e.to_string())?;

    let sessions_copied = copy_sessions(&old, &new)?;
    let messages_copied = copy_messages(&old, &new)?;

    let migrated = sessions_copied > 0 || messages_copied > 0;
    if migrated {
        info!(
            "[migrate_conversations] copied {sessions_copied} sessions, {messages_copied} messages \
             from {} to {}",
            old_path.display(),
            new_path.display()
        );
    }

    Ok(MigrationReport {
        migrated,
        sessions_copied,
        messages_copied,
    })
}
