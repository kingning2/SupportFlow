//! stdio 按行读取与 stderr 日志转发。

use std::sync::Arc;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::runtime::StdioJsonRpcRuntime;
use crate::task::ProcessTaskContext;

pub async fn read_line_lossy<R>(reader: &mut R, line: &mut String) -> std::io::Result<usize>
where
    R: AsyncBufReadExt + Unpin,
{
    let mut buf = Vec::new();
    let n = reader.read_until(b'\n', &mut buf).await?;
    if n == 0 {
        line.clear();
        return Ok(0);
    }
    *line = String::from_utf8_lossy(&buf).into_owned();
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }
    Ok(n)
}

pub fn forward_log_line(log_prefix: &str, trimmed: &str) {
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        if v.get("type").and_then(|t| t.as_str()) == Some("log") {
            let level = v.get("level").and_then(|l| l.as_str()).unwrap_or("info");
            let msg = v.get("msg").and_then(|m| m.as_str()).unwrap_or(trimmed);
            match level {
                "error" => tracing::error!(target: "process", "[{log_prefix}] {msg}"),
                "warning" | "warn" => tracing::warn!(target: "process", "[{log_prefix}] {msg}"),
                "debug" => tracing::debug!(target: "process", "[{log_prefix}] {msg}"),
                _ => tracing::info!(target: "process", "[{log_prefix}] {msg}"),
            }
            return;
        }
    }
    let name = ProcessTaskContext::current_process_name().unwrap_or(log_prefix);
    tracing::debug!(target: "process", "[{name}] {trimmed}");
}

pub fn spawn_stderr_reader(
    process_name: &'static str,
    log_prefix: &'static str,
    stderr: tokio::process::ChildStderr,
) {
    tokio::spawn(async move {
        ProcessTaskContext::scope(
            ProcessTaskContext { process_name },
            async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                while read_line_lossy(&mut reader, &mut line).await.unwrap_or(0) > 0 {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        forward_log_line(log_prefix, trimmed);
                    }
                    line.clear();
                }
            },
        )
        .await;
    });
}

pub async fn tokio_stdout_reader_loop(
    runtime: Arc<StdioJsonRpcRuntime>,
    reader: &mut BufReader<tokio::process::ChildStdout>,
) {
    let mut line = String::new();
    loop {
        line.clear();
        let n = match read_line_lossy(reader, &mut line).await {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(
                    target: "process",
                    "[{}] stdout reader ended: {e}",
                    runtime.process_name()
                );
                break;
            }
        };
        if n == 0 {
            runtime.clear_process_state().await;
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        runtime.dispatch_stdout_line(trimmed).await;
    }
}
