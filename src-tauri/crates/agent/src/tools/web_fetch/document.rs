//! Document URL helpers for web_fetch (`web_fetch.py` helpers).

use std::path::Path;

use url::Url;

use crate::knowledge::{
    is_supported_suffix, parse_document_file as knowledge_parse, MAX_INGEST_BYTES,
};
use crate::tools::utils::truncate::{format_size, truncate_head};

pub use crate::knowledge::is_supported_suffix as is_supported_doc_suffix;

pub const MAX_FILE_SIZE: usize = MAX_INGEST_BYTES;

pub fn url_suffix(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| {
            Path::new(u.path())
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        })
        .unwrap_or_default()
}

pub fn is_document_url(url: &str) -> bool {
    is_supported_suffix(&url_suffix(url))
}

pub fn is_binary_content_type(content_type: &str) -> bool {
    let ct = content_type.to_lowercase();
    [
        "application/pdf",
        "application/vnd.openxmlformats",
        "application/vnd.ms-excel",
        "application/vnd.ms-powerpoint",
        "application/octet-stream",
    ]
    .iter()
    .any(|p| ct.contains(p))
}

pub fn suffix_from_content_type(content_type: &str) -> Option<&'static str> {
    let ct = content_type.to_lowercase();
    if ct.contains("application/pdf") {
        return Some(".pdf");
    }
    if ct.contains("application/vnd.openxmlformats-officedocument.wordprocessingml") {
        return Some(".docx");
    }
    if ct.contains("application/vnd.ms-excel") {
        return Some(".xls");
    }
    if ct.contains("application/vnd.openxmlformats-officedocument.spreadsheetml") {
        return Some(".xlsx");
    }
    if ct.contains("application/vnd.ms-powerpoint") {
        return Some(".ppt");
    }
    if ct.contains("application/vnd.openxmlformats-officedocument.presentationml") {
        return Some(".pptx");
    }
    None
}

pub fn rewrite_url_with_suffix(url: &str, suffix: &str) -> String {
    let mut parsed = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return url.to_string(),
    };
    let path = parsed.path().trim_end_matches('/').to_string() + suffix;
    parsed.set_path(&path);
    parsed.to_string()
}

pub fn safe_filename(url: &str) -> String {
    let path = Url::parse(url)
        .map(|u| u.path().to_string())
        .unwrap_or_else(|_| "downloaded_file".into());
    let basename = Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty() && *s != "/")
        .unwrap_or("downloaded_file");
    let safe: String = basename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let id = uuid::Uuid::new_v4().simple().to_string();
    let id = &id[..8];
    format!("{id}_{safe}")
}

pub fn cleanup_file(path: &Path) {
    let _ = std::fs::remove_file(path);
}

pub fn parse_document_file(path: &Path, suffix: &str) -> Result<String, String> {
    knowledge_parse(path, Some(suffix))
}

pub fn format_document_result(filename: &str, local_path: &Path, text: &str) -> String {
    let file_size = std::fs::metadata(local_path).map(|m| m.len() as usize).unwrap_or(0);

    if text.trim().is_empty() {
        return format!(
            "File downloaded to: {} ({})\nNo text content could be extracted. The file may contain only images or be encrypted.",
            local_path.display(),
            format_size(file_size)
        );
    }

    let truncation = truncate_head(text, None, None);
    let mut header = format!(
        "[Document: {filename} | Size: {} | Saved to: {}]\n\n",
        format_size(file_size),
        local_path.display()
    );
    if truncation.truncated {
        header.push_str(&format!(
            "[Content truncated: showing {} of {} lines]\n\n",
            truncation.output_lines, truncation.total_lines
        ));
    }
    format!("{header}{}", truncation.content)
}
