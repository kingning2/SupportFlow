//! `agent/knowledge/ingest.py` — upload → Markdown pages + index/log.

use std::fs;
use std::path::Path;

use chrono::Local;
use regex::Regex;
use uuid::Uuid;

use crate::services::agent::tools::utils::truncate::truncate_head;

use super::document_parser::{
    all_doc_suffixes, is_supported_filename, parse_document_file, MAX_INGEST_BYTES,
};

pub const INGEST_MAX_LINES: usize = 8000;
pub const INGEST_MAX_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IngestResult {
    pub path: String,
    pub title: String,
    pub category: String,
    pub slug: String,
    pub original_name: String,
    pub truncated: bool,
    pub char_count: usize,
    pub archive: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IngestBatchResult {
    pub results: Vec<IngestResult>,
    pub errors: Vec<IngestError>,
    pub count: usize,
    #[serde(default)]
    pub memory_synced: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IngestError {
    pub file: String,
    pub message: String,
}

pub fn supported_suffixes_list() -> Vec<&'static str> {
    all_doc_suffixes()
}

fn slugify(name: &str) -> String {
    let base = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name)
        .trim()
        .to_lowercase();
    let re = Regex::new(r"[^\w\u{4e00}-\u{9fff}\-]+").expect("slug regex");
    let mut slug = re.replace_all(&base, "-").into_owned();
    let dash = Regex::new(r"-+").expect("dash regex");
    slug = dash.replace_all(&slug, "-").into_owned();
    slug = slug.trim_matches('-').to_string();
    if slug.len() > 80 {
        slug.truncate(80);
    }
    if slug.is_empty() {
        "document".into()
    } else {
        slug
    }
}

fn safe_category(category: &str) -> Result<String, String> {
    let mut category = category.trim().to_lowercase();
    if category.is_empty() {
        category = "uploads".into();
    }
    let re = Regex::new(r"[^\w\-]+").map_err(|e| e.to_string())?;
    category = re.replace_all(&category, "-").into_owned();
    category = category.trim_matches('-').to_string();
    if category.is_empty() {
        category = "uploads".into();
    }
    if matches!(category.as_str(), "." | ".." | "index" | "log" | "_sources") {
        return Err("invalid category name".into());
    }
    Ok(category)
}

fn unique_slug(knowledge_dir: &Path, category: &str, slug: &str) -> String {
    let rel = knowledge_dir.join(category).join(format!("{slug}.md"));
    if !rel.is_file() {
        return slug.to_string();
    }
    for i in 2..1000 {
        let candidate = format!("{slug}-{i}");
        let path = knowledge_dir.join(category).join(format!("{candidate}.md"));
        if !path.is_file() {
            return candidate;
        }
    }
    format!("{slug}-{}", &Uuid::new_v4().to_string()[..6])
}

fn read_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() > MAX_INGEST_BYTES {
        return Err(format!(
            "File too large (max {}MB)",
            MAX_INGEST_BYTES / (1024 * 1024)
        ));
    }
    Ok(data.to_vec())
}

/// Ingest one file into `knowledge/<category>/<slug>.md`.
pub fn ingest_bytes(
    knowledge_dir: &Path,
    filename: &str,
    source: &[u8],
    category: &str,
    knowledge_enabled: bool,
) -> Result<IngestResult, String> {
    if filename.is_empty() || !is_supported_filename(filename) {
        let ext_list = supported_suffixes_list().join(", ");
        return Err(format!("Unsupported file type. Supported: {ext_list}"));
    }
    if !knowledge_enabled {
        return Err("Knowledge base is disabled in config (knowledge=false)".into());
    }

    fs::create_dir_all(knowledge_dir).map_err(|e| e.to_string())?;
    let category = safe_category(category)?;
    let cat_dir = knowledge_dir.join(&category);
    fs::create_dir_all(&cat_dir).map_err(|e| e.to_string())?;

    let data = read_bytes(source)?;
    let ext = Path::new(filename)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    let sources_dir = knowledge_dir.join("_sources");
    fs::create_dir_all(&sources_dir).map_err(|e| e.to_string())?;
    let safe_name: String = Regex::new(r"[^\w.\-]+")
        .unwrap()
        .replace_all(
            Path::new(filename)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("file"),
            "_",
        )
        .into_owned();
    let safe_archive = format!("{}_{}", &Uuid::new_v4().to_string()[..8], safe_name);
    let archive_path = sources_dir.join(&safe_archive);
    fs::write(&archive_path, &data).map_err(|e| e.to_string())?;

    let text = parse_document_file(&archive_path, Some(&ext));
    let (body, truncated) = match text {
        Ok(t) if !t.trim().is_empty() => {
            let tr = truncate_head(&t, Some(INGEST_MAX_LINES), Some(INGEST_MAX_BYTES));
            (tr.content, tr.truncated)
        }
        Ok(_) => {
            let note = if ext == ".pdf" {
                "_(No text could be extracted from this PDF. It may be image-only/scanned; try OCR first, then upload again.)_"
            } else {
                "_(No text could be extracted from this file.)_"
            };
            (note.into(), false)
        }
        Err(e) => {
            tracing::warn!("[KnowledgeIngest] Parse failed for {filename}: {e}");
            (format!("_(Could not extract text: {e})_"), false)
        }
    };

    let title = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Uploaded document")
        .to_string();
    let slug = unique_slug(knowledge_dir, &category, &slugify(&title));
    let rel_path = format!("{category}/{slug}.md");

    let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let mut md = format!(
        "# {title}\n\n> Source: uploaded file `{filename}` ({now})\n> Archive: `_sources/{safe_archive}`\n\n"
    );
    if truncated {
        md.push_str("> Note: content was truncated to fit knowledge storage limits.\n\n");
    }
    md.push_str(&body);
    if !md.ends_with('\n') {
        md.push('\n');
    }

    let out_path = knowledge_dir.join(&rel_path);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&out_path, &md).map_err(|e| e.to_string())?;

    let mut summary = format!("uploaded from {filename}");
    if truncated {
        summary.push_str(" (truncated)");
    }
    append_index(knowledge_dir, &rel_path, &title, &summary)?;
    append_log(knowledge_dir, &rel_path, filename)?;

    tracing::info!("[KnowledgeIngest] Ingested {filename} -> {rel_path}");

    Ok(IngestResult {
        path: rel_path,
        title,
        category,
        slug,
        original_name: filename.to_string(),
        truncated,
        char_count: body.len(),
        archive: format!("_sources/{safe_archive}"),
    })
}

pub fn ingest_files(
    knowledge_dir: &Path,
    files: &[(String, Vec<u8>)],
    category: &str,
    knowledge_enabled: bool,
) -> IngestBatchResult {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    for (filename, data) in files {
        match ingest_bytes(knowledge_dir, filename, data, category, knowledge_enabled) {
            Ok(r) => results.push(r),
            Err(e) => errors.push(IngestError {
                file: filename.clone(),
                message: e,
            }),
        }
    }
    let count = results.len();
    IngestBatchResult {
        results,
        errors,
        count,
        memory_synced: false,
    }
}

fn append_index(
    knowledge_dir: &Path,
    rel_path: &str,
    title: &str,
    summary: &str,
) -> Result<(), String> {
    let index_path = knowledge_dir.join("index.md");
    if !index_path.is_file() {
        fs::write(&index_path, "# Knowledge Index\n\n").map_err(|e| e.to_string())?;
    }
    let mut content = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;

    let category = rel_path.split('/').next().unwrap_or("root");
    let section_title = category.replace('-', " ");
    let section_header = format!("## {}", title_case_section(&section_title));
    let line = format!("- [{title}]({rel_path}) — {summary}\n");

    if content.contains(&section_header) {
        let lines: Vec<String> = content.lines().map(str::to_string).collect();
        let mut out = Vec::new();
        let mut inserted = false;
        for line_text in lines {
            out.push(line_text.clone());
            if !inserted && line_text.trim() == section_header {
                out.push(line.trim_end().to_string());
                inserted = true;
            }
        }
        if !inserted {
            out.push(line.trim_end().to_string());
        }
        content = out.join("\n");
        if !content.ends_with('\n') {
            content.push('\n');
        }
    } else {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(&section_header);
        content.push('\n');
        content.push_str(&line);
    }

    fs::write(&index_path, content).map_err(|e| e.to_string())
}

fn title_case_section(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn append_log(knowledge_dir: &Path, rel_path: &str, original_name: &str) -> Result<(), String> {
    let log_path = knowledge_dir.join("log.md");
    let stamp = Local::now().format("%Y-%m-%d %H:%M");
    let entry = format!("{stamp} | ingest | {rel_path} | uploaded: {original_name}\n");
    if !log_path.is_file() {
        fs::write(&log_path, "# Knowledge Log\n\n").map_err(|e| e.to_string())?;
    }
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&log_path)
        .map_err(|e| e.to_string())?;
    f.write_all(entry.as_bytes()).map_err(|e| e.to_string())
}

/// Re-index `knowledge/*.md` for `memory_search` after ingest.
pub async fn trigger_memory_sync(
    workspace_root: &Path,
    models_config: &crate::config::ModelsConfig,
) -> bool {
    let enable_knowledge = models_config.knowledge.unwrap_or(true);
    if !enable_knowledge {
        return false;
    }
    match crate::services::agent::memory::create_memory_manager(
        workspace_root.to_path_buf(),
        models_config,
        enable_knowledge,
    ) {
        Ok(manager) => {
            manager.mark_dirty();
            match manager.sync().await {
                Ok(()) => true,
                Err(e) => {
                    tracing::warn!("[KnowledgeIngest] Memory sync skipped: {e}");
                    false
                }
            }
        }
        Err(e) => {
            tracing::warn!("[KnowledgeIngest] Memory sync skipped: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_strips_special_chars() {
        assert_eq!(slugify("Hello World!.md"), "hello-world");
    }

    #[test]
    fn safe_category_rejects_reserved() {
        assert!(safe_category("index").is_err());
        assert_eq!(safe_category("uploads").unwrap(), "uploads");
    }
}
