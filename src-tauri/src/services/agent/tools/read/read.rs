//! `agent/tools/read/read.py`

use std::path::Path;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::utils::path::is_supportflow_env_file;
use crate::services::agent::tools::utils::truncate::{
    format_size, truncate_head, DEFAULT_MAX_BYTES, DEFAULT_MAX_LINES,
};
use crate::services::agent::tools::workspace::WorkspaceToolConfig;

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

fn ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|s| format!(".{}", s.to_lowercase()))
        .unwrap_or_default()
}

fn is_image(ext: &str) -> bool {
    matches!(
        ext,
        ".jpg" | ".jpeg" | ".png" | ".gif" | ".webp" | ".bmp" | ".svg" | ".ico"
    )
}

fn is_video(ext: &str) -> bool {
    matches!(
        ext,
        ".mp4" | ".avi" | ".mov" | ".mkv" | ".flv" | ".wmv" | ".webm" | ".m4v"
    )
}

fn is_audio(ext: &str) -> bool {
    matches!(
        ext,
        ".mp3" | ".wav" | ".ogg" | ".m4a" | ".flac" | ".aac" | ".wma"
    )
}

fn is_binary_or_archive(ext: &str) -> bool {
    matches!(
        ext,
        ".exe"
            | ".dll"
            | ".so"
            | ".dylib"
            | ".bin"
            | ".dat"
            | ".db"
            | ".sqlite"
            | ".zip"
            | ".tar"
            | ".gz"
            | ".rar"
            | ".7z"
            | ".bz2"
            | ".xz"
    )
}

fn is_pdf(ext: &str) -> bool {
    ext == ".pdf"
}

fn is_office(ext: &str) -> bool {
    matches!(ext, ".doc" | ".docx" | ".xls" | ".xlsx" | ".ppt" | ".pptx")
}

pub struct ReadTool {
    config: WorkspaceToolConfig,
    description: String,
}

impl ReadTool {
    pub fn new(config: WorkspaceToolConfig) -> Self {
        let description = format!(
            "Read or inspect file contents. For text/PDF files, returns content (truncated to {DEFAULT_MAX_LINES} lines or {}KB). For images/videos/audio, returns metadata only (file info, size, type). Use offset/limit for large text files.",
            DEFAULT_MAX_BYTES / 1024
        );
        Self {
            config,
            description,
        }
    }

    fn file_not_found(&self, path: &str, absolute: &Path) -> ToolRunResult {
        if !Path::new(path).is_absolute() && !path.starts_with('~') {
            return ToolRunResult::error(format!(
                "Error: File not found: {path}\nResolved to: {}\nHint: Relative paths are based on workspace ({}). For files outside workspace, use absolute paths.",
                absolute.display(),
                self.config.display_cwd().display()
            ));
        }
        ToolRunResult::error(format!("Error: File not found: {path}"))
    }

    fn metadata_result(
        &self,
        absolute: &Path,
        file_type: &str,
        file_size: u64,
        message: String,
    ) -> ToolRunResult {
        let ext = ext_lower(absolute);
        let mime = match ext.as_str() {
            ".mp4" => "video/mp4",
            ".avi" => "video/x-msvideo",
            ".mov" => "video/quicktime",
            ".mkv" => "video/x-matroska",
            ".webm" => "video/webm",
            ".mp3" => "audio/mpeg",
            ".wav" => "audio/wav",
            ".ogg" => "audio/ogg",
            ".m4a" => "audio/mp4",
            ".flac" => "audio/flac",
            ".zip" => "application/zip",
            ".tar" => "application/x-tar",
            ".gz" => "application/gzip",
            ".rar" => "application/x-rar-compressed",
            _ => "application/octet-stream",
        };
        ToolRunResult::success(json!({
            "type": format!("{file_type}_metadata"),
            "file_type": file_type,
            "path": absolute.display().to_string(),
            "file_name": absolute.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "mime_type": mime,
            "size": file_size,
            "size_formatted": format_size(file_size as usize),
            "message": message,
        }))
    }

    fn read_image(&self, absolute: &Path, ext: &str) -> ToolRunResult {
        let file_size = std::fs::metadata(absolute).map(|m| m.len()).unwrap_or(0);
        let mime = match ext {
            ".jpg" | ".jpeg" => "image/jpeg",
            ".png" => "image/png",
            ".gif" => "image/gif",
            ".webp" => "image/webp",
            _ => "image/jpeg",
        };
        ToolRunResult::success(json!({
            "type": "image_metadata",
            "file_type": "image",
            "path": absolute.display().to_string(),
            "mime_type": mime,
            "size": file_size,
            "size_formatted": format_size(file_size as usize),
            "message": format!(
                "图片文件: {} ({})\n提示: 如果需要发送此图片，请使用 send 工具。",
                absolute.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                format_size(file_size as usize)
            ),
        }))
    }

    fn apply_offset_limit(
        content: &str,
        offset: Option<i64>,
        limit: Option<u64>,
    ) -> Result<(String, usize, usize, Option<usize>), String> {
        let all_lines: Vec<&str> = content.split('\n').collect();
        let total_file_lines = all_lines.len();

        let mut start_line = 0usize;
        if let Some(off) = offset {
            if off < 0 {
                start_line = total_file_lines.saturating_sub((-off) as usize);
            } else {
                start_line = (off as usize).saturating_sub(1);
                if start_line >= total_file_lines {
                    return Err(format!(
                        "Error: Offset {off} is beyond end of file ({total_file_lines} lines total)"
                    ));
                }
            }
        }

        let start_line_display = start_line + 1;
        let (selected_content, user_limited_lines) = if let Some(lim) = limit {
            let lim = lim as usize;
            let end_line = (start_line + lim).min(total_file_lines);
            (
                all_lines[start_line..end_line].join("\n"),
                Some(end_line - start_line),
            )
        } else if offset.is_some() {
            (all_lines[start_line..].join("\n"), None)
        } else {
            (content.to_string(), None)
        };

        Ok((
            selected_content,
            total_file_lines,
            start_line_display,
            user_limited_lines,
        ))
    }

    fn format_text_output(
        selected_content: &str,
        all_lines: &[&str],
        total_file_lines: usize,
        start_line: usize,
        start_line_display: usize,
        user_limited_lines: Option<usize>,
        display_path: &str,
    ) -> (String, Option<Value>) {
        let truncation = truncate_head(selected_content, None, None);
        let mut output_text: String;
        let mut details = None;

        if truncation.first_line_exceeds_limit {
            let first_line_size =
                format_size(all_lines.get(start_line).map(|l| l.len()).unwrap_or(0));
            output_text = format!(
                "[Line {start_line_display} is {first_line_size}, exceeds {} limit. Use bash tool to read: head -c {DEFAULT_MAX_BYTES} {display_path} | tail -n +{start_line_display}]",
                format_size(DEFAULT_MAX_BYTES)
            );
            details = Some(json!({ "truncation": truncation.to_value() }));
        } else if truncation.truncated {
            let end_line_display = start_line_display + truncation.output_lines.saturating_sub(1);
            let next_offset = end_line_display + 1;
            output_text = truncation.content.clone();
            if truncation.truncated_by.as_deref() == Some("lines") {
                output_text.push_str(&format!(
                    "\n\n[Showing lines {start_line_display}-{end_line_display} of {total_file_lines}. Use offset={next_offset} to continue.]"
                ));
            } else {
                output_text.push_str(&format!(
                    "\n\n[Showing lines {start_line_display}-{end_line_display} of {total_file_lines} ({} limit). Use offset={next_offset} to continue.]",
                    format_size(DEFAULT_MAX_BYTES)
                ));
            }
            details = Some(json!({ "truncation": truncation.to_value() }));
        } else if let Some(ull) = user_limited_lines {
            if start_line + ull < total_file_lines {
                let remaining = total_file_lines - (start_line + ull);
                let next_offset = start_line + ull + 1;
                output_text = truncation.content.clone();
                output_text.push_str(&format!(
                    "\n\n[{remaining} more lines in file. Use offset={next_offset} to continue.]"
                ));
            } else {
                output_text = truncation.content.clone();
            }
        } else {
            output_text = truncation.content.clone();
        }

        (output_text, details)
    }

    fn read_document(
        &self,
        absolute: &Path,
        display_path: &str,
        ext: &str,
        offset: Option<i64>,
        limit: Option<u64>,
    ) -> ToolRunResult {
        let text = match crate::services::agent::knowledge::parse_document_file(absolute, Some(ext))
        {
            Ok(t) => t,
            Err(e) => return ToolRunResult::error(format!("Error reading document: {e}")),
        };
        if text.trim().is_empty() {
            let name = absolute
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            return ToolRunResult::success(json!({
                "content": format!("[Office file {name}: no text content could be extracted]"),
            }));
        }
        self.read_text_content(&text, display_path, offset, limit)
    }

    fn read_text(
        &self,
        absolute: &Path,
        display_path: &str,
        offset: Option<i64>,
        limit: Option<u64>,
    ) -> ToolRunResult {
        let meta = match std::fs::metadata(absolute) {
            Ok(m) => m,
            Err(e) => return ToolRunResult::error(format!("Error reading file: {e}")),
        };
        if meta.len() > MAX_FILE_SIZE {
            return ToolRunResult::success(json!({
                "type": "file_to_send",
                "file_type": "document",
                "path": absolute.display().to_string(),
                "size": meta.len(),
                "size_formatted": format_size(meta.len() as usize),
                "message": format!(
                    "文件过大 ({} > 50MB)，无法读取内容。文件路径: {}",
                    format_size(meta.len() as usize),
                    absolute.display()
                ),
            }));
        }

        let content = match std::fs::read_to_string(absolute) {
            Ok(s) => {
                if let Some(rest) = s.strip_prefix('\u{feff}') {
                    rest.to_string()
                } else {
                    s
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                return ToolRunResult::error(format!(
                    "Error: File is not a valid text file (encoding error): {display_path}"
                ));
            }
            Err(e) => return ToolRunResult::error(format!("Error reading file: {e}")),
        };

        self.read_text_content(&content, display_path, offset, limit)
    }

    fn read_text_content(
        &self,
        content: &str,
        display_path: &str,
        offset: Option<i64>,
        limit: Option<u64>,
    ) -> ToolRunResult {
        let all_lines: Vec<&str> = content.split('\n').collect();
        let (selected, total_file_lines, start_line_display, user_limited) =
            match Self::apply_offset_limit(content, offset, limit) {
                Ok(v) => v,
                Err(msg) => return ToolRunResult::error(msg),
            };
        let start_line = start_line_display.saturating_sub(1);
        let (output_text, details) = Self::format_text_output(
            &selected,
            &all_lines,
            total_file_lines,
            start_line,
            start_line_display,
            user_limited,
            display_path,
        );

        let mut result = json!({
            "content": output_text,
            "total_lines": total_file_lines,
            "start_line": start_line_display,
            "output_lines": output_text.lines().count(),
        });
        if let Some(d) = details {
            result
                .as_object_mut()
                .expect("obj")
                .insert("details".into(), d);
        }
        ToolRunResult::success(result)
    }

    async fn execute_inner(&self, args: &Value) -> ToolRunResult {
        let path = args
            .get("path")
            .or_else(|| args.get("location"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let offset = args.get("offset").and_then(|v| v.as_i64());
        let limit = args.get("limit").and_then(|v| v.as_u64());

        if path.is_empty() {
            return ToolRunResult::error("Error: path parameter is required");
        }

        let absolute = self.config.resolve(path);
        if is_supportflow_env_file(&absolute) {
            return ToolRunResult::error(
                "Error: Access denied. API keys and credentials must be accessed through the env_config tool only.",
            );
        }

        if !absolute.exists() {
            return self.file_not_found(path, &absolute);
        }

        let ext = ext_lower(&absolute);
        let file_size = std::fs::metadata(&absolute).map(|m| m.len()).unwrap_or(0);

        if is_image(&ext) {
            return self.read_image(&absolute, &ext);
        }
        if is_video(&ext) {
            return self.metadata_result(
                &absolute,
                "video",
                file_size,
                format!(
                    "Video 文件: {} ({})\n提示: 如果需要发送此文件，请使用 send 工具。",
                    absolute.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    format_size(file_size as usize)
                ),
            );
        }
        if is_audio(&ext) {
            return self.metadata_result(
                &absolute,
                "audio",
                file_size,
                format!(
                    "Audio 文件: {} ({})\n提示: 如果需要发送此文件，请使用 send 工具。",
                    absolute.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    format_size(file_size as usize)
                ),
            );
        }
        if is_binary_or_archive(&ext) {
            return self.metadata_result(
                &absolute,
                "binary",
                file_size,
                format!(
                    "Binary 文件: {} ({})\n提示: 如果需要发送此文件，请使用 send 工具。",
                    absolute.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    format_size(file_size as usize)
                ),
            );
        }
        if is_pdf(&ext) || is_office(&ext) {
            return self.read_document(&absolute, path, &ext, offset, limit);
        }

        self.read_text(&absolute, path, offset, limit)
    }
}

#[async_trait]
impl AgentTool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read. IMPORTANT: Relative paths are based on workspace directory."
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (1-indexed). Negative reads from end."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        self.execute_inner(&params).await
    }
}
