//! SQLite storage with FTS5 + vector search (`agent/memory/storage.py`).

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub id: String,
    pub user_id: Option<String>,
    pub scope: String,
    pub source: String,
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub text: String,
    pub embedding: Option<Vec<f32>>,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub score: f64,
    pub snippet: String,
    pub source: String,
    pub user_id: Option<String>,
}

pub struct MemoryStorage {
    conn: Mutex<Connection>,
    fts5_available: bool,
}

impl MemoryStorage {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;",
        )
        .map_err(|e| e.to_string())?;

        let fts5_available = Self::check_fts5(&conn)?;
        let storage = Self {
            conn: Mutex::new(conn),
            fts5_available,
        };
        storage.init_schema()?;
        Ok(storage)
    }

    fn check_fts5(conn: &Connection) -> Result<bool, String> {
        match conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS fts5_test USING fts5(test);
             DROP TABLE IF EXISTS fts5_test;",
        ) {
            Ok(()) => Ok(true),
            Err(e) if e.to_string().contains("fts5") => Ok(false),
            Err(e) => Err(e.to_string()),
        }
    }

    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                scope TEXT NOT NULL DEFAULT 'shared',
                source TEXT NOT NULL DEFAULT 'memory',
                path TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                text TEXT NOT NULL,
                embedding BLOB,
                hash TEXT NOT NULL,
                metadata TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER DEFAULT (strftime('%s', 'now'))
            );
            CREATE INDEX IF NOT EXISTS idx_chunks_user ON chunks(user_id);
            CREATE INDEX IF NOT EXISTS idx_chunks_scope ON chunks(scope);
            CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                source TEXT NOT NULL DEFAULT 'memory',
                hash TEXT NOT NULL,
                mtime INTEGER NOT NULL,
                size INTEGER NOT NULL,
                updated_at INTEGER DEFAULT (strftime('%s', 'now'))
            );",
        )
        .map_err(|e| e.to_string())?;

        if self.fts5_available {
            let _ = conn.execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                    text,
                    id UNINDEXED,
                    user_id UNINDEXED,
                    path UNINDEXED,
                    source UNINDEXED,
                    scope UNINDEXED,
                    content='chunks',
                    content_rowid='rowid'
                );
                CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
                    INSERT INTO chunks_fts(rowid, text, id, user_id, path, source, scope)
                    VALUES (new.rowid, new.text, new.id, new.user_id, new.path, new.source, new.scope);
                END;
                CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON chunks BEGIN
                    DELETE FROM chunks_fts WHERE rowid = old.rowid;
                END;
                CREATE TRIGGER IF NOT EXISTS chunks_au AFTER UPDATE ON chunks BEGIN
                    UPDATE chunks_fts SET text = new.text, id = new.id,
                        user_id = new.user_id, path = new.path,
                        source = new.source, scope = new.scope
                    WHERE rowid = new.rowid;
                END;",
            );
        }

        Ok(())
    }

    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn get_file_hash(&self, path: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT hash FROM files WHERE path = ?1")
            .map_err(|e| e.to_string())?;
        let hash = stmt
            .query_row(params![path], |row| row.get::<_, String>(0))
            .ok();
        Ok(hash)
    }

    pub fn delete_by_path(&self, path: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM chunks WHERE path = ?1", params![path])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save_chunks_batch(&self, chunks: &[MemoryChunk]) -> Result<(), String> {
        if chunks.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO chunks
                    (id, user_id, scope, source, path, start_line, end_line, text, embedding, hash, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, strftime('%s', 'now'))
                    ON CONFLICT(id) DO UPDATE SET
                        user_id = excluded.user_id,
                        scope = excluded.scope,
                        source = excluded.source,
                        path = excluded.path,
                        start_line = excluded.start_line,
                        end_line = excluded.end_line,
                        text = excluded.text,
                        embedding = excluded.embedding,
                        hash = excluded.hash,
                        updated_at = strftime('%s', 'now')",
                )
                .map_err(|e| e.to_string())?;
            for chunk in chunks {
                stmt.execute(params![
                    chunk.id,
                    chunk.user_id,
                    chunk.scope,
                    chunk.source,
                    chunk.path,
                    chunk.start_line,
                    chunk.end_line,
                    chunk.text,
                    encode_embedding(chunk.embedding.as_deref()),
                    chunk.hash,
                ])
                .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_file_metadata(
        &self,
        path: &str,
        source: &str,
        file_hash: &str,
        mtime: i64,
        size: i64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO files (path, source, hash, mtime, size, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, strftime('%s', 'now'))",
            params![path, source, file_hash, mtime, size],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn search_vector(
        &self,
        query_embedding: &[f32],
        user_id: Option<&str>,
        scopes: &[&str],
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        if scopes.is_empty() {
            return Ok(vec![]);
        }

        let scope_list: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();
        let scope_ph = scope_list
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let sql = if user_id.is_some() {
            format!(
                "SELECT path, start_line, end_line, text, source, user_id, embedding
                 FROM chunks
                 WHERE scope IN ({scope_ph})
                 AND (scope = 'shared' OR user_id = ?)
                 AND embedding IS NOT NULL"
            )
        } else {
            format!(
                "SELECT path, start_line, end_line, text, source, user_id, embedding
                 FROM chunks
                 WHERE scope IN ({scope_ph})
                 AND embedding IS NOT NULL"
            )
        };

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = if let Some(uid) = user_id {
            let mut params: Vec<&dyn rusqlite::types::ToSql> = scope_list
                .iter()
                .map(|s| s as &dyn rusqlite::types::ToSql)
                .collect();
            params.push(&uid);
            stmt.query_map(params.as_slice(), row_to_vector_parts)
        } else {
            let params: Vec<&dyn rusqlite::types::ToSql> = scope_list
                .iter()
                .map(|s| s as &dyn rusqlite::types::ToSql)
                .collect();
            stmt.query_map(params.as_slice(), row_to_vector_parts)
        }
        .map_err(|e| e.to_string())?;

        let q_norm = vector_norm(query_embedding);
        let mut scored: Vec<(f64, SearchResult)> = Vec::new();

        for row in rows.flatten() {
            let Some(blob) = row.6 else { continue };
            let Some(vec) = decode_embedding(&blob) else {
                continue;
            };
            if vec.len() != query_embedding.len() {
                continue;
            }
            let sim = cosine_similarity(query_embedding, &vec, q_norm);
            if sim <= 0.0 {
                continue;
            }
            scored.push((
                sim,
                SearchResult {
                    path: row.0.clone(),
                    start_line: row.1 as u32,
                    end_line: row.2 as u32,
                    score: sim,
                    snippet: truncate_text(&row.3, 500),
                    source: row.4.clone(),
                    user_id: row.5.clone(),
                },
            ));
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        Ok(scored.into_iter().map(|(_, r)| r).collect())
    }

    pub fn search_keyword(
        &self,
        query: &str,
        user_id: Option<&str>,
        scopes: &[&str],
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        if scopes.is_empty() {
            return Ok(vec![]);
        }

        if self.fts5_available && !contains_cjk(query) {
            if let Some(fts_query) = build_fts_query(query) {
                let results = self.search_fts5(&fts_query, user_id, scopes, limit)?;
                if !results.is_empty() {
                    return Ok(results);
                }
            }
        }

        // LIKE fallback: FTS5 unavailable, CJK queries, or FTS5 returned no hits.
        self.search_like(query, user_id, scopes, limit)
    }

    fn search_fts5(
        &self,
        fts_query: &str,
        user_id: Option<&str>,
        scopes: &[&str],
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let scope_list: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();
        let scope_ph = scope_list
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let lim = limit as i64;

        let sql = if user_id.is_some() {
            format!(
                "SELECT chunks.path, chunks.start_line, chunks.end_line, chunks.text,
                        chunks.source, chunks.user_id, bm25(chunks_fts) as rank
                 FROM chunks_fts
                 JOIN chunks ON chunks.rowid = chunks_fts.rowid
                 WHERE chunks_fts MATCH ?1
                 AND chunks.scope IN ({scope_ph})
                 AND (chunks.scope = 'shared' OR chunks.user_id = ?)
                 ORDER BY rank
                 LIMIT ?"
            )
        } else {
            format!(
                "SELECT chunks.path, chunks.start_line, chunks.end_line, chunks.text,
                        chunks.source, chunks.user_id, bm25(chunks_fts) as rank
                 FROM chunks_fts
                 JOIN chunks ON chunks.rowid = chunks_fts.rowid
                 WHERE chunks_fts MATCH ?1
                 AND chunks.scope IN ({scope_ph})
                 ORDER BY rank
                 LIMIT ?"
            )
        };

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

        if let Some(uid) = user_id {
            let mut params: Vec<&dyn rusqlite::types::ToSql> =
                vec![&fts_query as &dyn rusqlite::types::ToSql];
            for s in &scope_list {
                params.push(s);
            }
            params.push(&uid);
            params.push(&lim);
            let rows = stmt
                .query_map(params.as_slice(), row_to_search_result)
                .map_err(|e| e.to_string())?;
            return Ok(rows.flatten().collect());
        }

        let mut params: Vec<&dyn rusqlite::types::ToSql> =
            vec![&fts_query as &dyn rusqlite::types::ToSql];
        for s in &scope_list {
            params.push(s);
        }
        params.push(&lim);
        let rows = stmt
            .query_map(params.as_slice(), row_to_search_result)
            .map_err(|e| e.to_string())?;
        Ok(rows.flatten().collect())
    }

    fn search_like(
        &self,
        query: &str,
        user_id: Option<&str>,
        scopes: &[&str],
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let words = extract_like_tokens(query);
        if words.is_empty() {
            return Ok(vec![]);
        }

        let scope_ph = scopes.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let like_parts: Vec<String> = words
            .iter()
            .map(|_| "LOWER(text) LIKE ?".to_string())
            .collect();
        let where_clause = like_parts.join(" OR ");

        let sql = if user_id.is_some() {
            format!(
                "SELECT path, start_line, end_line, text, source, user_id FROM chunks
                 WHERE ({where_clause})
                 AND scope IN ({scope_ph})
                 AND (scope = 'shared' OR user_id = ?)
                 LIMIT ?"
            )
        } else {
            format!(
                "SELECT path, start_line, end_line, text, source, user_id FROM chunks
                 WHERE ({where_clause})
                 AND scope IN ({scope_ph})
                 LIMIT ?"
            )
        };

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

        let mut bind: Vec<Box<dyn rusqlite::types::ToSql>> = words
            .iter()
            .map(|w| Box::new(format!("%{}%", w.to_lowercase())) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        for s in scopes {
            bind.push(Box::new(s.to_string()));
        }
        if let Some(uid) = user_id {
            bind.push(Box::new(uid.to_string()));
        }
        bind.push(Box::new(limit as i64));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            bind.iter().map(|b| b.as_ref()).collect();
        let rows = stmt
            .query_map(params_ref.as_slice(), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for row in rows.flatten() {
            let text_lower = row.3.to_lowercase();
            let matched = words
                .iter()
                .filter(|w| text_lower.contains(w.as_str()))
                .count();
            if matched == 0 {
                continue;
            }
            let score = (0.3 + 0.15 * matched as f64).min(0.85);
            results.push(SearchResult {
                path: row.0,
                start_line: row.1 as u32,
                end_line: row.2 as u32,
                score,
                snippet: truncate_text(&row.3, 500),
                source: row.4,
                user_id: row.5,
            });
        }
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        Ok(results)
    }
}

fn encode_embedding(embedding: Option<&[f32]>) -> Option<Vec<u8>> {
    embedding.map(|e| e.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>())
}

fn decode_embedding(raw: &[u8]) -> Option<Vec<f32>> {
    if !raw.len().is_multiple_of(4) {
        return None;
    }
    Some(
        raw.chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}

fn vector_norm(v: &[f32]) -> f64 {
    (v.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>()).sqrt()
}

fn cosine_similarity(a: &[f32], b: &[f32], a_norm: f64) -> f64 {
    let b_norm = vector_norm(b).max(1e-10);
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as f64) * (*y as f64))
        .sum();
    dot / (a_norm.max(1e-10) * b_norm)
}

fn bm25_rank_to_score(rank: f64) -> f64 {
    0.3 + 0.69 * (rank.abs() / (1.0 + rank.abs()))
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let s: String = text.chars().take(max_chars).collect();
    format!("{s}...")
}

fn build_fts_query(raw: &str) -> Option<String> {
    let tokens: Vec<String> = regex::Regex::new(r"[A-Za-z0-9_]+")
        .ok()?
        .find_iter(raw)
        .map(|m| format!("\"{}\"", m.as_str()))
        .collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" OR "))
    }
}

fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c,
            '\u{3000}'..='\u{30ff}'
            | '\u{3400}'..='\u{9fff}'
            | '\u{ac00}'..='\u{d7af}'
            | '\u{f900}'..='\u{faff}'
        )
    })
}

type VectorRowParts = (
    String,
    i64,
    i64,
    String,
    String,
    Option<String>,
    Option<Vec<u8>>,
);

fn row_to_vector_parts(row: &rusqlite::Row<'_>) -> rusqlite::Result<VectorRowParts> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
    ))
}

fn row_to_search_result(row: &rusqlite::Row<'_>) -> rusqlite::Result<SearchResult> {
    Ok(SearchResult {
        path: row.get(0)?,
        start_line: row.get::<_, i64>(1)? as u32,
        end_line: row.get::<_, i64>(2)? as u32,
        score: bm25_rank_to_score(row.get(6)?),
        snippet: truncate_text(&row.get::<_, String>(3)?, 500),
        source: row.get(4)?,
        user_id: row.get(5)?,
    })
}

fn extract_like_tokens(query: &str) -> Vec<String> {
    let mut out = Vec::new();
    let cjk_re = regex::Regex::new(
        r"[\u{3000}-\u{30ff}\u{3400}-\u{9fff}\u{ac00}-\u{d7af}\u{f900}-\u{faff}]+",
    )
    .ok();
    if let Some(re) = cjk_re {
        for m in re.find_iter(query) {
            out.push(m.as_str().to_string());
        }
    }
    if let Ok(re) = regex::Regex::new(r"[A-Za-z0-9_]+") {
        for m in re.find_iter(query) {
            let t = m.as_str();
            if t.len() >= 3 {
                out.push(t.to_lowercase());
            }
        }
    }
    out
}
