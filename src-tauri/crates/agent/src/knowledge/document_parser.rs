//! `agent/knowledge/document_parser.py` — local document → Markdown (MarkItDown) then legacy fallbacks.

use std::io::Read;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};
use pdf_extract::extract_text;
use zip::ZipArchive;

use super::markitdown;

pub const MAX_INGEST_BYTES: usize = 50 * 1024 * 1024;

pub const PDF_SUFFIXES: &[&str] = &[".pdf"];
pub const WORD_SUFFIXES: &[&str] = &[".docx"];
pub const TEXT_SUFFIXES: &[&str] = &[
    ".txt", ".md", ".markdown", ".rst", ".csv", ".tsv", ".log", ".json", ".xml", ".html", ".htm",
];
pub const SPREADSHEET_SUFFIXES: &[&str] = &[".xls", ".xlsx"];
pub const PPT_SUFFIXES: &[&str] = &[".ppt", ".pptx"];
/// Extra formats handled when MarkItDown is installed (images, archives, notebooks, …).
pub const MARKITDOWN_EXTRA_SUFFIXES: &[&str] = &[
    ".jpeg", ".jpg", ".png", ".gif", ".webp", ".bmp", ".tif", ".tiff", ".zip", ".epub", ".ipynb",
    ".wav", ".mp3", ".m4a",
];

pub fn all_doc_suffixes() -> Vec<&'static str> {
    let mut v = Vec::new();
    v.extend_from_slice(PDF_SUFFIXES);
    v.extend_from_slice(WORD_SUFFIXES);
    v.extend_from_slice(TEXT_SUFFIXES);
    v.extend_from_slice(SPREADSHEET_SUFFIXES);
    v.extend_from_slice(PPT_SUFFIXES);
    v.extend_from_slice(MARKITDOWN_EXTRA_SUFFIXES);
    v
}

fn suffix_in_set(suffix: &str, set: &[&str]) -> bool {
    set.contains(&suffix)
}

pub fn is_supported_suffix(suffix: &str) -> bool {
    all_doc_suffixes().contains(&suffix)
}

pub fn is_supported_filename(filename: &str) -> bool {
    let ext = Path::new(filename)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    is_supported_suffix(&ext)
}

pub fn parse_document_file(file_path: impl AsRef<Path>, suffix: Option<&str>) -> Result<String, String> {
    let path = file_path.as_ref();
    let suffix = suffix
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| {
            path.extension()
                .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
                .unwrap_or_default()
        });

    match markitdown::convert_file_to_markdown(path) {
        Ok(text) if !text.trim().is_empty() => return Ok(text),
        Ok(_) => tracing::warn!(
            "[parser] MarkItDown returned empty ({} bytes file)",
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        ),
        Err(e) => tracing::warn!("[parser] MarkItDown skipped: {e}"),
    }

    // Legacy .doc (non-OXML) is not a ZIP package, so the Rust fallback
    // docx parser will produce misleading "invalid Zip archive" errors.
    // We intentionally only support .docx in the Rust fallback path.
    if suffix == ".doc" {
        return Err("Legacy .doc is not supported by the Rust parser; please convert/export it to .docx (or upload as PDF).".into());
    }

    if suffix_in_set(&suffix, PDF_SUFFIXES) {
        return parse_pdf(path);
    }
    if suffix_in_set(&suffix, WORD_SUFFIXES) {
        return parse_word(path);
    }
    if suffix_in_set(&suffix, SPREADSHEET_SUFFIXES) {
        return parse_spreadsheet(path);
    }
    if suffix_in_set(&suffix, PPT_SUFFIXES) {
        return parse_ppt(path);
    }
    parse_text(path)
}

fn parse_pdf(path: &Path) -> Result<String, String> {
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let text = extract_text(path).map_err(|e| format!("parse PDF: {e}"))?;
    if text.trim().is_empty() {
        tracing::warn!(
            "[parser] pdf-extract returned empty ({} bytes file — likely scanned/image PDF)",
            file_size
        );
        return Ok(String::new());
    }
    Ok(text)
}

fn parse_word(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open docx: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("docx zip: {e}"))?;
    let mut entry = archive
        .by_name("word/document.xml")
        .map_err(|e| format!("word/document.xml: {e}"))?;
    let mut xml = String::new();
    entry
        .read_to_string(&mut xml)
        .map_err(|e| format!("read document.xml: {e}"))?;

    let re = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").map_err(|e| e.to_string())?;
    let para_re = regex::Regex::new(r"</w:p>").map_err(|e| e.to_string())?;
    let mut paragraphs: Vec<String> = Vec::new();
    for block in para_re.split(&xml) {
        let text: String = re
            .captures_iter(block)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str())
            .collect();
        let text = text.trim();
        if !text.is_empty() {
            paragraphs.push(text.to_string());
        }
    }
    Ok(paragraphs.join("\n\n"))
}

fn parse_text(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read file: {e}"))?;
    for label in ["utf-8", "utf-8-sig", "gbk", "gb2312", "latin-1"] {
        let norm = label.trim_end_matches("-sig");
        if let Some(encoding) = encoding_rs::Encoding::for_label(norm.as_bytes()) {
            let (text, _, had_errors) = encoding.decode(&bytes);
            if !had_errors || label != "utf-8" {
                return Ok(text.into_owned());
            }
        }
    }
    Err(format!("Unable to decode file: {}", path.display()))
}

fn parse_spreadsheet(path: &Path) -> Result<String, String> {
    let mut workbook =
        open_workbook_auto(path).map_err(|e| format!("open spreadsheet: {e}"))?;
    let sheet_names = workbook.sheet_names().to_vec();
    let mut parts = Vec::new();

    for sheet_name in sheet_names {
        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|e| format!("read sheet {sheet_name}: {e}"))?;
        let mut rows = Vec::new();
        for row in range.rows() {
            let cells: Vec<String> = row
                .iter()
                .map(|c| match c {
                    Data::Empty => String::new(),
                    Data::String(s) => s.clone(),
                    Data::Float(f) => f.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(d) => d.to_string(),
                    Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
                    Data::Error(e) => format!("{e:?}"),
                })
                .collect();
            if cells.iter().any(|c| !c.is_empty()) {
                rows.push(cells.join(" | "));
            }
        }
        if !rows.is_empty() {
            parts.push(format!("--- Sheet: {sheet_name} ---\n{}", rows.join("\n")));
        }
    }
    Ok(parts.join("\n\n"))
}

fn parse_ppt(path: &Path) -> Result<String, String> {
    let suffix = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if suffix == "ppt" {
        return Err(
            "Legacy .ppt format is not supported; convert to .pptx or use browser.".into(),
        );
    }
    parse_pptx_zip(path)
}

fn parse_pptx_zip(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open pptx: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("pptx zip: {e}"))?;
    let re = regex::Regex::new(r"(?s)<a:t[^>]*>([^<]*)</a:t>").map_err(|e| e.to_string())?;

    let mut slide_files: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let entry = archive.by_index(i).ok()?;
            let name = entry.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    slide_files.sort();

    let mut parts = Vec::new();
    for (idx, name) in slide_files.iter().enumerate() {
        let mut entry = archive.by_name(name).map_err(|e| format!("slide {name}: {e}"))?;
        let mut xml = String::new();
        entry
            .read_to_string(&mut xml)
            .map_err(|e| format!("read slide xml: {e}"))?;
        let texts: Vec<String> = re
            .captures_iter(&xml)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !texts.is_empty() {
            let total = slide_files.len();
            parts.push(format!(
                "--- Slide {}/{total} ---\n{}",
                idx + 1,
                texts.join("\n")
            ));
        }
    }
    Ok(parts.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_suffix_set_matches_python() {
        assert!(is_supported_suffix(".pdf"));
        assert!(is_supported_suffix(".docx"));
        assert!(is_supported_suffix(".xlsx"));
        assert!(is_supported_suffix(".pptx"));
    }
}
