//! High-level memory manager (`agent/memory/manager.py`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;

use super::chunker::{TextChunk, TextChunker};
use super::config::MemoryConfig;
use super::embedding::EmbeddingProvider;
use super::storage::{MemoryChunk, MemoryStorage, SearchResult};
use crate::{MemoryManager, MemorySearchHit};

pub struct DbMemoryManager {
    config: MemoryConfig,
    storage: Arc<MemoryStorage>,
    chunker: TextChunker,
    embedding: Option<Arc<dyn EmbeddingProvider>>,
    embedding_cache: std::sync::Mutex<HashMap<String, Vec<f32>>>,
    dirty: AtomicBool,
}

impl DbMemoryManager {
    pub fn new(
        config: MemoryConfig,
        storage: Arc<MemoryStorage>,
        embedding: Option<Arc<dyn EmbeddingProvider>>,
    ) -> Self {
        if let Some(parent) = config.memory_dir().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::create_dir_all(config.memory_dir());
        Self {
            chunker: TextChunker::new(config.chunk_max_tokens, config.chunk_overlap_tokens),
            config,
            storage,
            embedding,
            embedding_cache: std::sync::Mutex::new(HashMap::new()),
            dirty: AtomicBool::new(true),
        }
    }

    pub fn set_dirty(&self) {
        self.dirty.store(true, Ordering::SeqCst);
    }

    fn generate_chunk_id(path: &str, start_line: u32, end_line: u32) -> String {
        format!("{:x}", md5::compute(format!("{path}:{start_line}:{end_line}")))
    }

    fn scopes_for_search(&self, user_id: Option<&str>) -> Vec<String> {
        let mut scopes = vec!["shared".to_string()];
        if user_id.is_some() {
            scopes.push("user".to_string());
        }
        scopes
    }

    fn merge_results(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<SearchResult>,
    ) -> Vec<SearchResult> {
        let mut merged: HashMap<(String, u32, u32), (SearchResult, f64, f64)> = HashMap::new();

        for r in vector_results {
            let key = (r.path.clone(), r.start_line, r.end_line);
            let score = r.score;
            merged.insert(key, (r, score, 0.0));
        }
        for r in keyword_results {
            let key = (r.path.clone(), r.start_line, r.end_line);
            let ks = r.score;
            if let Some(entry) = merged.get_mut(&key) {
                entry.2 = ks;
            } else {
                merged.insert(key, (r, 0.0, ks));
            }
        }

        let mut out: Vec<SearchResult> = merged
            .into_values()
            .map(|(mut result, vs, ks)| {
                // Keyword-only hits use full keyword score so FTS/LIKE matches are not
                // scaled below min_score when embeddings are disabled or miss the chunk.
                let base = if vs > 0.0 {
                    self.config.vector_weight * vs + self.config.keyword_weight * ks
                } else {
                    ks
                };
                result.score = base * temporal_decay(&result.path);
                result
            })
            .collect();
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }

    fn rel_path_from_workspace(workspace: &Path, file_path: &Path) -> String {
        let workspace = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());
        let file_path = file_path.canonicalize().unwrap_or_else(|_| file_path.to_path_buf());
        file_path
            .strip_prefix(&workspace)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| file_path.to_string_lossy().replace('\\', "/"))
    }

    pub async fn sync_index(&self) -> Result<(), String> {
        let workspace = self.config.workspace().to_path_buf();
        let mut files_to_scan: Vec<(PathBuf, String, String, Option<String>)> = Vec::new();

        let memory_md = workspace.join("MEMORY.md");
        if memory_md.is_file() {
            files_to_scan.push((memory_md, "memory".into(), "shared".into(), None));
        }

        let memory_dir = self.config.memory_dir();
        if memory_dir.is_dir() {
            collect_md_files(&memory_dir, &workspace, &mut files_to_scan)?;
        }

        if self.config.enable_knowledge {
            let knowledge_dir = workspace.join("knowledge");
            if knowledge_dir.is_dir() {
                for path in walk_md_paths(&knowledge_dir) {
                    files_to_scan.push((path, "knowledge".into(), "shared".into(), None));
                }
            }
        }

        struct PendingFile {
            rel_path: String,
            source: String,
            scope: String,
            user_id: Option<String>,
            file_hash: String,
            chunks: Vec<TextChunk>,
            texts: Vec<String>,
            mtime: i64,
            size: i64,
        }

        let mut pending: Vec<PendingFile> = Vec::new();

        for (file_path, source, scope, user_id) in files_to_scan {
            let content = match std::fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let rel_path = Self::rel_path_from_workspace(&workspace, &file_path);
            if rel_path.split('/').any(|p| p.starts_with('.')) {
                continue;
            }
            if rel_path.contains("/dreams/") || rel_path.starts_with("dreams/") {
                continue;
            }
            let file_hash = MemoryStorage::compute_hash(&content);
            if self.storage.get_file_hash(&rel_path)? == Some(file_hash.clone()) {
                continue;
            }
            let chunks = self.chunker.chunk_text(&content);
            if chunks.is_empty() {
                continue;
            }
            let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
            let meta = std::fs::metadata(&file_path).ok();
            let mtime = meta
                .as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let size = meta.map(|m| m.len() as i64).unwrap_or(0);
            pending.push(PendingFile {
                rel_path,
                source,
                scope,
                user_id,
                file_hash,
                chunks,
                texts,
                mtime,
                size,
            });
        }

        if pending.is_empty() {
            self.dirty.store(false, Ordering::SeqCst);
            return Ok(());
        }

        let all_texts: Vec<String> = pending.iter().flat_map(|p| p.texts.clone()).collect();
        let all_embeddings: Vec<Option<Vec<f32>>> = if let Some(provider) = &self.embedding {
            match provider.embed_batch(&all_texts).await {
                Ok(v) => v.into_iter().map(Some).collect(),
                Err(e) => {
                    tracing::warn!(
                        "[MemoryManager] batch embedding failed ({} chunks): {e}",
                        all_texts.len()
                    );
                    return Err(e);
                }
            }
        } else {
            vec![None; all_texts.len()]
        };

        let mut cursor = 0usize;
        for entry in pending {
            let n = entry.texts.len();
            let entry_embeddings = &all_embeddings[cursor..cursor + n];
            cursor += n;

            self.storage.delete_by_path(&entry.rel_path)?;
            let mut memory_chunks = Vec::with_capacity(entry.chunks.len());
            for (chunk, embedding) in entry.chunks.iter().zip(entry_embeddings.iter()) {
                let chunk_id =
                    Self::generate_chunk_id(&entry.rel_path, chunk.start_line, chunk.end_line);
                let chunk_hash = MemoryStorage::compute_hash(&chunk.text);
                memory_chunks.push(MemoryChunk {
                    id: chunk_id,
                    user_id: entry.user_id.clone(),
                    scope: entry.scope.clone(),
                    source: entry.source.clone(),
                    path: entry.rel_path.clone(),
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    text: chunk.text.clone(),
                    embedding: embedding.clone(),
                    hash: chunk_hash,
                });
            }
            self.storage.save_chunks_batch(&memory_chunks)?;
            self.storage.update_file_metadata(
                &entry.rel_path,
                &entry.source,
                &entry.file_hash,
                entry.mtime,
                entry.size,
            )?;
        }

        self.dirty.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn cached_embed_query(&self, query: &str, provider: &Arc<dyn EmbeddingProvider>) -> Result<Vec<f32>, String> {
        let cache_key = format!(
            "{}:{}:{}",
            provider.provider_name(),
            provider.model_name(),
            query
        );
        if let Ok(guard) = self.embedding_cache.lock() {
            if let Some(vec) = guard.get(&cache_key) {
                return Ok(vec.clone());
            }
        }
        let vec = provider.embed_query(query).await?;
        if let Ok(mut guard) = self.embedding_cache.lock() {
            guard.insert(cache_key, vec.clone());
        }
        Ok(vec)
    }
}

fn collect_md_files(
    dir: &Path,
    workspace: &Path,
    out: &mut Vec<(PathBuf, String, String, Option<String>)>,
) -> Result<(), String> {
    for path in walk_md_paths(dir) {
        let rel_path = DbMemoryManager::rel_path_from_workspace(workspace, &path);
        if rel_path.split('/').any(|p| p.starts_with('.')) {
            continue;
        }
        if rel_path.contains("/dreams/") || rel_path.starts_with("dreams/") {
            continue;
        }
        let rel_parts: Vec<String> = rel_path.split('/').map(|s| s.to_string()).collect();
        let (scope, user_id) = classify_memory_path(&rel_parts);
        out.push((path, "memory".into(), scope, user_id));
    }
    Ok(())
}

fn classify_memory_path(parts: &[String]) -> (String, Option<String>) {
    if parts.iter().any(|p| p == "daily") {
        if let Some(idx) = parts.iter().position(|p| p == "users") {
            let user_id = parts.get(idx + 1).cloned();
            return ("user".into(), user_id);
        }
        return ("shared".into(), None);
    }
    if let Some(idx) = parts.iter().position(|p| p == "users") {
        let user_id = parts.get(idx + 1).cloned();
        return ("user".into(), user_id);
    }
    ("shared".into(), None)
}

fn walk_md_paths(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_md_inner(dir, &mut out);
    out
}

fn walk_md_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_md_inner(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn sync_indexes_workspace_memory_md() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ws = dir.path().to_path_buf();
        std::fs::write(ws.join("MEMORY.md"), "user prefers dark mode\n").expect("write");

        let cfg = MemoryConfig::new(&ws);
        let storage = Arc::new(MemoryStorage::open(&cfg.db_path()).expect("open db"));
        let manager = DbMemoryManager::new(cfg, storage.clone(), None);
        manager.sync_index().await.expect("sync");

        let hits = storage
            .search_keyword("dark mode", None, &["shared"], 5)
            .expect("search");
        assert!(!hits.is_empty(), "expected indexed chunks");

        let mgr_hits = manager
            .search("dark mode", None, 10, 0.1)
            .await
            .expect("manager search");
        assert!(!mgr_hits.is_empty(), "manager hybrid search should return hits");
    }
}

fn temporal_decay(path: &str) -> f64 {
    let re = regex::Regex::new(r"(\d{4})-(\d{2})-(\d{2})\.md$").ok();
    let Some(re) = re else {
        return 1.0;
    };
    let Some(caps) = re.captures(path) else {
        return 1.0;
    };
    let y: i32 = caps[1].parse().unwrap_or(0);
    let m: u32 = caps[2].parse().unwrap_or(0);
    let d: u32 = caps[3].parse().unwrap_or(0);
    let file_date = chrono::NaiveDate::from_ymd_opt(y, m, d);
    let Some(file_date) = file_date else {
        return 1.0;
    };
    let today = chrono::Local::now().date_naive();
    let age_days = (today - file_date).num_days();
    if age_days <= 0 {
        return 1.0;
    }
    let half_life = 30.0_f64;
    let decay_lambda = std::f64::consts::LN_2 / half_life;
    (-decay_lambda * age_days as f64).exp()
}

#[async_trait]
impl MemoryManager for DbMemoryManager {
    fn workspace(&self) -> &Path {
        self.config.workspace()
    }

    async fn search(
        &self,
        query: &str,
        user_id: Option<&str>,
        max_results: usize,
        min_score: f64,
    ) -> Result<Vec<MemorySearchHit>, String> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        if self.config.sync_on_search && self.dirty.load(Ordering::SeqCst) {
            self.sync_index().await?;
        }

        let scopes = self.scopes_for_search(user_id);
        let scope_refs: Vec<&str> = scopes.iter().map(|s| s.as_str()).collect();
        let limit = max_results * 2;

        let mut vector_results = Vec::new();
        if let Some(provider) = &self.embedding {
            if let Ok(query_embedding) = self.cached_embed_query(query, provider).await {
                vector_results = self.storage.search_vector(
                    &query_embedding,
                    user_id,
                    &scope_refs,
                    limit,
                )?;
            }
        }

        let keyword_results = self
            .storage
            .search_keyword(query, user_id, &scope_refs, limit)?;

        if keyword_results.is_empty() && vector_results.is_empty() {
            return Ok(vec![]);
        }

        let merged = self.merge_results(vector_results, keyword_results);
        Ok(merged
            .into_iter()
            .filter(|r| r.score >= min_score)
            .take(max_results)
            .map(|r| MemorySearchHit {
                path: r.path,
                start_line: r.start_line,
                end_line: r.end_line,
                score: r.score,
                snippet: r.snippet,
            })
            .collect())
    }

    fn mark_dirty(&self) {
        self.set_dirty();
    }

    async fn sync(&self) -> Result<(), String> {
        self.sync_index().await
    }
}
