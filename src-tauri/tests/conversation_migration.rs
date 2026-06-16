//! Conversation DB split and legacy migration tests.

use rusqlite::Connection;
use tauri_app_lib::services::agent::memory::{
    migrate_conversations_for_workspace, ConversationStore, MemoryConfig,
};

const LEGACY_DDL: &str = r"
CREATE TABLE sessions (
    session_id        TEXT    PRIMARY KEY,
    channel_type      TEXT    NOT NULL DEFAULT '',
    title             TEXT    NOT NULL DEFAULT '',
    context_start_seq INTEGER NOT NULL DEFAULT 0,
    created_at        INTEGER NOT NULL,
    last_active       INTEGER NOT NULL,
    msg_count         INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE messages (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id   TEXT    NOT NULL,
    seq          INTEGER NOT NULL,
    role         TEXT    NOT NULL,
    content      TEXT    NOT NULL,
    created_at   INTEGER NOT NULL,
    extras       TEXT    NOT NULL DEFAULT '',
    UNIQUE (session_id, seq)
);
";

fn seed_legacy_db(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create legacy db parent");
    }
    let conn = Connection::open(path).expect("open legacy db");
    conn.execute_batch(LEGACY_DDL).expect("legacy ddl");
    conn.execute(
        "INSERT INTO sessions
            (session_id, channel_type, title, context_start_seq, created_at, last_active, msg_count)
         VALUES ('sess-1', 'terminal', 'Hello', 0, 100, 200, 2)",
        [],
    )
    .expect("insert session");
    conn.execute(
        "INSERT INTO messages (session_id, seq, role, content, created_at, extras)
         VALUES ('sess-1', 0, 'user', '\"hi\"', 100, '')",
        [],
    )
    .expect("insert msg 0");
    conn.execute(
        "INSERT INTO messages (session_id, seq, role, content, created_at, extras)
         VALUES ('sess-1', 1, 'assistant', '\"hello back\"', 101, '')",
        [],
    )
    .expect("insert msg 1");
}

fn count_rows(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .expect("count")
}

#[test]
fn conversation_db_path_is_separate_from_memory_db() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = MemoryConfig::new(dir.path());
    assert_eq!(cfg.db_path(), dir.path().join("memory/long-term/index.db"));
    assert_eq!(
        cfg.conversation_db_path(),
        dir.path().join("conversations/index.db")
    );
    assert_ne!(cfg.db_path(), cfg.conversation_db_path());
}

#[test]
fn migrate_legacy_sessions_and_messages_idempotently() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ws = dir.path();
    let cfg = MemoryConfig::new(ws);

    seed_legacy_db(&cfg.db_path());

    let first = migrate_conversations_for_workspace(ws).expect("first migrate");
    assert!(first.migrated);
    assert_eq!(first.sessions_copied, 1);
    assert_eq!(first.messages_copied, 2);

    let store = ConversationStore::open(&cfg.conversation_db_path()).expect("open new db");
    let messages = store
        .load_messages("sess-1", 10)
        .expect("load migrated messages");
    assert_eq!(messages.len(), 2);
    assert_eq!(
        messages[0].get("role").and_then(|v| v.as_str()),
        Some("user")
    );

    let second = migrate_conversations_for_workspace(ws).expect("second migrate");
    assert!(!second.migrated);
    assert_eq!(second.sessions_copied, 0);
    assert_eq!(second.messages_copied, 0);

    let new_conn = Connection::open(cfg.conversation_db_path()).expect("open new conn");
    assert_eq!(count_rows(&new_conn, "sessions"), 1);
    assert_eq!(count_rows(&new_conn, "messages"), 2);
}

#[test]
fn new_install_uses_conversation_db_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ws = dir.path();
    let cfg = MemoryConfig::new(ws);

    let store = ConversationStore::for_workspace(ws).expect("open store");
    store
        .append_messages(
            "fresh",
            &[serde_json::json!({"role": "user", "content": "test"})],
            "terminal",
        )
        .expect("append");

    assert!(cfg.conversation_db_path().is_file());
    assert!(!cfg.db_path().is_file());

    let loaded = store.load_messages("fresh", 5).expect("load");
    assert_eq!(loaded.len(), 1);
}
