//! 日志文件路径解析与后台 tail 流。

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Emitter};

use crate::events::names::AGENT_LOG_STREAM;
use crate::events::payloads::AgentLogStreamPayload;

use super::AgentRuntime;

fn resolve_latest_log_path() -> Result<PathBuf, String> {
    let dirs = directories::ProjectDirs::from("com", "polymerization", "gybte")
        .ok_or_else(|| "could not resolve app log directory".to_string())?;
    let log_root = dirs.data_local_dir().join("logs");
    let date = crate::utils::date::current_date_string();
    Ok(log_root.join(format!("tauri-app.{date}.log")))
}

fn latest_lines_from(path: &PathBuf, limit: usize) -> Result<String, String> {
    let raw = crate::utils::fs::read_to_string(path)?;
    let lines: Vec<&str> = raw.lines().collect();
    let start = lines.len().saturating_sub(limit);
    Ok(lines[start..].join("\n"))
}

impl AgentRuntime {
    /// Enable or disable background log streaming flag.
    pub async fn set_log_streaming(&self, value: bool) {
        *self.log_streaming.write().await = value;
    }

    /// Read background log streaming flag.
    pub async fn log_streaming(&self) -> bool {
        *self.log_streaming.read().await
    }

    /// Resolve latest tauri log source and return `(enabled, source_path)`.
    pub async fn logs_status(&self) -> Result<(bool, String), String> {
        let source = resolve_latest_log_path()?;
        Ok((source.exists(), source.display().to_string()))
    }

    /// Start background log tailing and emit `agent/log-stream` events.
    pub async fn start_log_stream(self: Arc<Self>, app: AppHandle) -> Result<bool, String> {
        let source = resolve_latest_log_path()?;
        if !source.exists() {
            app.emit(
                AGENT_LOG_STREAM,
                AgentLogStreamPayload {
                    payload_type: "error".to_string(),
                    content: None,
                    message: Some("log file not found".to_string()),
                },
            )
            .map_err(|e| e.to_string())?;
            return Ok(false);
        }

        self.set_log_streaming(true).await;
        let app_handle = app.clone();
        let runtime_ref = self.clone();
        let source_path = source.clone();

        tokio::spawn(async move {
            let init = latest_lines_from(&source_path, 500).unwrap_or_default();
            let _ = app_handle.emit(
                AGENT_LOG_STREAM,
                AgentLogStreamPayload {
                    payload_type: "init".to_string(),
                    content: Some(init.clone()),
                    message: None,
                },
            );

            let mut previous_len = init.len();
            let mut last_modified = fs::metadata(&source_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            while runtime_ref.log_streaming().await {
                tokio::time::sleep(Duration::from_millis(900)).await;

                let meta = match fs::metadata(&source_path) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                if modified <= last_modified {
                    continue;
                }
                last_modified = modified;

                let text = match fs::read_to_string(&source_path) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if text.len() < previous_len {
                    previous_len = text.len();
                    let _ = app_handle.emit(
                        AGENT_LOG_STREAM,
                        AgentLogStreamPayload {
                            payload_type: "init".to_string(),
                            content: Some(text),
                            message: None,
                        },
                    );
                    continue;
                }

                let mut slice_start = previous_len.min(text.len());
                while slice_start > 0 && !text.is_char_boundary(slice_start) {
                    slice_start -= 1;
                }
                let delta = text[slice_start..].to_string();
                previous_len = text.len();
                if delta.trim().is_empty() {
                    continue;
                }

                let _ = app_handle.emit(
                    AGENT_LOG_STREAM,
                    AgentLogStreamPayload {
                        payload_type: "line".to_string(),
                        content: Some(delta),
                        message: None,
                    },
                );
            }
        });

        Ok(true)
    }

    /// Stop background log tailing loop.
    pub async fn stop_log_stream(&self) {
        self.set_log_streaming(false).await;
    }

    /// Read latest log lines with optional line limit.
    pub async fn read_logs(&self, limit: Option<i32>) -> Result<(String, String), String> {
        let source = resolve_latest_log_path()?;
        if !source.exists() {
            return Ok((source.display().to_string(), String::new()));
        }

        let raw = crate::utils::fs::read_to_string(&source)?;
        let limit = limit.and_then(|v| usize::try_from(v).ok()).unwrap_or(400);
        let lines: Vec<&str> = raw.lines().collect();
        let start = lines.len().saturating_sub(limit);
        let content = lines[start..].join("\n");
        Ok((source.display().to_string(), content))
    }
}
