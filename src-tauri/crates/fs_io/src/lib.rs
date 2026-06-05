//! Logged `std::fs` wrappers — single implementation for desktop app, CLI, and libraries.
//!
//! All operations emit structured `[fs] action=…` lines via `tracing`.
//! Callers map `std::io::Error` at IPC boundaries (`Result<_, String>`) as needed.

use std::borrow::Cow;
use std::path::Path;

const PREVIEW_LIMIT: usize = 240;

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

fn summarize_bytes(bytes: &[u8]) -> String {
    match String::from_utf8_lossy(bytes) {
        Cow::Borrowed(text) => summarize_text(text),
        Cow::Owned(text) => {
            if text.contains('\u{fffd}') {
                summarize_binary(bytes)
            } else {
                summarize_text(&text)
            }
        }
    }
}

fn summarize_text(text: &str) -> String {
    let sanitized = text.replace('\r', "\\r").replace('\n', "\\n");
    if sanitized.chars().count() > PREVIEW_LIMIT {
        let preview: String = sanitized.chars().take(PREVIEW_LIMIT).collect();
        format!("{preview}...(truncated)")
    } else {
        sanitized
    }
}

fn summarize_binary(bytes: &[u8]) -> String {
    let preview = bytes
        .iter()
        .take(24)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    if bytes.len() > 24 {
        format!("hex[{preview}]...(truncated)")
    } else {
        format!("hex[{preview}]")
    }
}

fn log_success(action: &str, path: &Path, detail: &str) {
    tracing::info!("[fs] action={action} path={} {detail}", display_path(path));
}

fn log_failure(action: &str, path: &Path, error: &std::io::Error) {
    tracing::warn!(
        "[fs] action={action} path={} error={}",
        display_path(path),
        error
    );
}

fn log_write_change(path: &Path, before: Option<&[u8]>, after: &[u8]) {
    let before_summary = before
        .map(summarize_bytes)
        .unwrap_or_else(|| "<missing>".to_string());
    let after_summary = summarize_bytes(after);
    tracing::info!(
        "[fs] action=write path={} before_bytes={} after_bytes={} before={} after={}",
        display_path(path),
        before.map(|v| v.len()).unwrap_or(0),
        after.len(),
        before_summary,
        after_summary
    );
}

/// 递归创建目录。
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    match std::fs::create_dir_all(path) {
        Ok(()) => {
            log_success("create_dir_all", path, "status=ok");
            Ok(())
        }
        Err(error) => {
            log_failure("create_dir_all", path, &error);
            Err(error)
        }
    }
}

/// 读取文本文件。
pub fn read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let path = path.as_ref();
    match std::fs::read_to_string(path) {
        Ok(content) => {
            log_success(
                "read_to_string",
                path,
                &format!("bytes={} preview={}", content.len(), summarize_text(&content)),
            );
            Ok(content)
        }
        Err(error) => {
            log_failure("read_to_string", path, &error);
            Err(error)
        }
    }
}

/// 读取二进制文件。
pub fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
    let path = path.as_ref();
    match std::fs::read(path) {
        Ok(bytes) => {
            log_success(
                "read",
                path,
                &format!("bytes={} preview={}", bytes.len(), summarize_bytes(&bytes)),
            );
            Ok(bytes)
        }
        Err(error) => {
            log_failure("read", path, &error);
            Err(error)
        }
    }
}

/// 写入文件（覆盖）。
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
    let path = path.as_ref();
    let after = contents.as_ref();
    let before = std::fs::read(path).ok();
    match std::fs::write(path, after) {
        Ok(()) => {
            log_write_change(path, before.as_deref(), after);
            Ok(())
        }
        Err(error) => {
            log_failure("write", path, &error);
            Err(error)
        }
    }
}

/// 删除文件。
pub fn remove_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    let before = std::fs::read(path).ok();
    match std::fs::remove_file(path) {
        Ok(()) => {
            let before_summary = before
                .as_deref()
                .map(summarize_bytes)
                .unwrap_or_else(|| "<missing>".to_string());
            log_success(
                "remove_file",
                path,
                &format!(
                    "removed_bytes={} removed_preview={}",
                    before.as_ref().map(|v| v.len()).unwrap_or(0),
                    before_summary
                ),
            );
            Ok(())
        }
        Err(error) => {
            log_failure("remove_file", path, &error);
            Err(error)
        }
    }
}

/// 复制文件。
pub fn copy<F: AsRef<Path>, T: AsRef<Path>>(from: F, to: T) -> std::io::Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();
    match std::fs::copy(from, to) {
        Ok(bytes) => {
            tracing::info!(
                "[fs] action=copy from={} to={} bytes={}",
                display_path(from),
                display_path(to),
                bytes
            );
            Ok(bytes)
        }
        Err(error) => {
            tracing::warn!(
                "[fs] action=copy from={} to={} error={}",
                display_path(from),
                display_path(to),
                error
            );
            Err(error)
        }
    }
}

/// 读取路径元数据。
pub fn metadata<P: AsRef<Path>>(path: P) -> std::io::Result<std::fs::Metadata> {
    let path = path.as_ref();
    match std::fs::metadata(path) {
        Ok(metadata) => {
            log_success(
                "metadata",
                path,
                &format!("is_file={} len={}", metadata.is_file(), metadata.len()),
            );
            Ok(metadata)
        }
        Err(error) => {
            log_failure("metadata", path, &error);
            Err(error)
        }
    }
}

/// 枚举目录项。
pub fn read_dir<P: AsRef<Path>>(path: P) -> std::io::Result<std::fs::ReadDir> {
    let path = path.as_ref();
    match std::fs::read_dir(path) {
        Ok(entries) => {
            log_success("read_dir", path, "status=ok");
            Ok(entries)
        }
        Err(error) => {
            log_failure("read_dir", path, &error);
            Err(error)
        }
    }
}
