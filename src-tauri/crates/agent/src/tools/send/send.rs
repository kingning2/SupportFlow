//! `agent/tools/send/send.py`

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::tools::base_tool::{AgentTool, ToolRunResult};
use crate::tools::send::upload::SendFileUploader;
use crate::tools::utils::truncate::format_size;
use crate::tools::workspace::WorkspaceToolConfig;

fn ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|s| format!(".{}", s.to_lowercase()))
        .unwrap_or_default()
}

pub struct SendTool {
    config: WorkspaceToolConfig,
    uploader: Arc<dyn SendFileUploader>,
}

impl SendTool {
    pub fn new(config: WorkspaceToolConfig, uploader: Arc<dyn SendFileUploader>) -> Self {
        Self { config, uploader }
    }

    fn image_mime(ext: &str) -> &'static str {
        match ext {
            ".jpg" | ".jpeg" => "image/jpeg",
            ".png" => "image/png",
            ".gif" => "image/gif",
            ".webp" => "image/webp",
            ".bmp" => "image/bmp",
            ".svg" => "image/svg+xml",
            ".ico" => "image/x-icon",
            _ => "image/jpeg",
        }
    }

    fn video_mime(ext: &str) -> &'static str {
        match ext {
            ".mp4" => "video/mp4",
            ".avi" => "video/x-msvideo",
            ".mov" => "video/quicktime",
            ".mkv" => "video/x-matroska",
            ".webm" => "video/webm",
            ".flv" => "video/x-flv",
            _ => "video/mp4",
        }
    }

    fn audio_mime(ext: &str) -> &'static str {
        match ext {
            ".mp3" => "audio/mpeg",
            ".wav" => "audio/wav",
            ".ogg" => "audio/ogg",
            ".m4a" => "audio/mp4",
            ".flac" => "audio/flac",
            ".aac" => "audio/aac",
            _ => "audio/mpeg",
        }
    }

    fn document_mime(ext: &str) -> &'static str {
        match ext {
            ".pdf" => "application/pdf",
            ".doc" => "application/msword",
            ".docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ".xls" => "application/vnd.ms-excel",
            ".xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ".ppt" => "application/vnd.ms-powerpoint",
            ".pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ".txt" => "text/plain",
            ".md" => "text/markdown",
            _ => "application/octet-stream",
        }
    }
}

#[async_trait]
impl AgentTool for SendTool {
    fn name(&self) -> &str {
        "send"
    }

    fn description(&self) -> &str {
        "Send a LOCAL file (image, video, audio, document) to the user. Only for local file paths. Do NOT use this for URLs — URLs should be included directly in your text reply, the system will handle them automatically."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Local file path to send. Must be absolute or relative to workspace. Do NOT pass URLs."
                },
                "message": {
                    "type": "string",
                    "description": "Optional message to accompany the file"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let message = params
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if path.is_empty() {
            return ToolRunResult::error("Error: path parameter is required");
        }

        if path.starts_with("http://") || path.starts_with("https://") {
            return ToolRunResult::error(
                "Error: URLs cannot be sent with the send tool. Include the URL in your text reply instead.",
            );
        }

        let absolute = self.config.resolve(path);
        if !absolute.exists() {
            return ToolRunResult::error(format!("Error: File not found: {path}"));
        }

        let meta = match std::fs::metadata(&absolute) {
            Ok(m) => m,
            Err(e) => return ToolRunResult::error(format!("Error: {e}")),
        };
        if !meta.is_file() {
            return ToolRunResult::error(format!("Error: Not a file: {path}"));
        }

        let ext = ext_lower(&absolute);
        let file_size = meta.len();
        let file_name = absolute
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let (file_type, mime_type) = if matches!(
            ext.as_str(),
            ".jpg" | ".jpeg" | ".png" | ".gif" | ".webp" | ".bmp" | ".svg" | ".ico"
        ) {
            ("image", Self::image_mime(&ext))
        } else if matches!(
            ext.as_str(),
            ".mp4" | ".avi" | ".mov" | ".mkv" | ".flv" | ".wmv" | ".webm" | ".m4v"
        ) {
            ("video", Self::video_mime(&ext))
        } else if matches!(
            ext.as_str(),
            ".mp3" | ".wav" | ".ogg" | ".m4a" | ".flac" | ".aac" | ".wma"
        ) {
            ("audio", Self::audio_mime(&ext))
        } else if matches!(
            ext.as_str(),
            ".pdf" | ".doc" | ".docx" | ".xls" | ".xlsx" | ".ppt" | ".pptx" | ".txt" | ".md"
        ) {
            ("document", Self::document_mime(&ext))
        } else {
            ("file", "application/octet-stream")
        };

        let default_msg = format!("正在发送 {file_name}");
        let mut result = json!({
            "type": "file_to_send",
            "file_type": file_type,
            "path": absolute.display().to_string(),
            "file_name": file_name,
            "mime_type": mime_type,
            "size": file_size,
            "size_formatted": format_size(file_size as usize),
            "message": if message.is_empty() { default_msg } else { message },
        });

        if let Some(url) = self
            .uploader
            .copy_send_file(&absolute, self.config.display_cwd())
            .await
        {
            result
                .as_object_mut()
                .expect("obj")
                .insert("url".into(), Value::String(url));
        }

        ToolRunResult::success(result)
    }
}
