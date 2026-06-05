//! 渠道收件箱 SQLite 持久化与快照（跨 Webview 共享）。

use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessageDto {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_name: Option<String>,
    pub content: String,
    pub created_at: i64,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelConversationSummaryDto {
    pub conversation_id: String,
    pub session_id: String,
    pub title: String,
    pub kind: String,
    pub last_active: i64,
    pub preview: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unread: Option<i32>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInboxSnapshotDto {
    pub conversations: Vec<ChannelConversationSummaryDto>,
    pub messages: Vec<ChannelMessageDto>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInboxMessagePayload {
    pub channel: String,
    pub message: ChannelMessageDto,
}

pub struct ChannelInboxStore {
    conn: Mutex<Connection>,
}

impl ChannelInboxStore {
    pub fn open(app: &AppHandle) -> Result<Self, String> {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        crate::utils::fs::create_dir_all(&dir)?;
        let path = dir.join("channel_inbox.db");
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                channel TEXT NOT NULL,
                conversation_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                title TEXT NOT NULL,
                kind TEXT NOT NULL,
                last_active INTEGER NOT NULL,
                preview TEXT NOT NULL,
                unread INTEGER,
                PRIMARY KEY (channel, conversation_id)
            );
            CREATE TABLE IF NOT EXISTS messages (
                channel TEXT NOT NULL,
                id TEXT NOT NULL,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                sender_name TEXT,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (channel, id)
            );
            "#,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn snapshot(&self, channel: Option<&str>) -> Result<ChannelInboxSnapshotDto, String> {
        let guard = crate::utils::err::lock_mutex(&self.conn)?;
        let conversations = load_conversations(&guard, channel)?;
        let messages = load_messages(&guard, channel)?;
        Ok(ChannelInboxSnapshotDto {
            conversations,
            messages,
        })
    }
}

fn load_conversations(
    conn: &Connection,
    channel: Option<&str>,
) -> Result<Vec<ChannelConversationSummaryDto>, String> {
    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<ChannelConversationSummaryDto> {
        Ok(ChannelConversationSummaryDto {
            conversation_id: row.get(0)?,
            session_id: row.get(1)?,
            title: row.get(2)?,
            kind: row.get(3)?,
            last_active: row.get(4)?,
            preview: row.get(5)?,
            unread: row.get(6)?,
        })
    };

    let mut out = Vec::new();
    match channel {
        Some(ch) if !ch.is_empty() => {
            let mut stmt = conn
                .prepare(
                    "SELECT conversation_id, session_id, title, kind, last_active, preview, unread
                     FROM conversations WHERE channel = ?1 ORDER BY last_active DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![ch], map_row)
                .map_err(|e| e.to_string())?;
            for row in rows {
                out.push(row.map_err(|e| e.to_string())?);
            }
        }
        _ => {
            let mut stmt = conn
                .prepare(
                    "SELECT conversation_id, session_id, title, kind, last_active, preview, unread
                     FROM conversations ORDER BY last_active DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], map_row).map_err(|e| e.to_string())?;
            for row in rows {
                out.push(row.map_err(|e| e.to_string())?);
            }
        }
    }
    Ok(out)
}

fn load_messages(
    conn: &Connection,
    channel: Option<&str>,
) -> Result<Vec<ChannelMessageDto>, String> {
    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<ChannelMessageDto> {
        Ok(ChannelMessageDto {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            sender_name: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get(5)?,
        })
    };

    let mut out = Vec::new();
    match channel {
        Some(ch) if !ch.is_empty() => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, conversation_id, role, sender_name, content, created_at
                     FROM messages WHERE channel = ?1 ORDER BY created_at ASC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![ch], map_row)
                .map_err(|e| e.to_string())?;
            for row in rows {
                out.push(row.map_err(|e| e.to_string())?);
            }
        }
        _ => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, conversation_id, role, sender_name, content, created_at
                     FROM messages ORDER BY created_at ASC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], map_row).map_err(|e| e.to_string())?;
            for row in rows {
                out.push(row.map_err(|e| e.to_string())?);
            }
        }
    }
    Ok(out)
}
