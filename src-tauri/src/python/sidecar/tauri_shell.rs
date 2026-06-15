//! Tauri `externalBin` shell sidecar 适配层（仅 desktop）。

use std::sync::Arc;

use crate::process_runtime::{StdinLineWriter, StdioJsonRpcRuntime};
use async_trait::async_trait;
use tauri_plugin_shell::process::CommandEvent;
use tokio::sync::Mutex;

/// Tauri shell 子进程 stdin 写入器。
pub struct ShellStdinWriter {
    child: Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
}

impl ShellStdinWriter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            child: Mutex::new(None),
        })
    }

    pub async fn set_child(&self, child: tauri_plugin_shell::process::CommandChild) {
        *self.child.lock().await = Some(child);
    }

    pub async fn clear(&self) {
        *self.child.lock().await = None;
    }
}

#[async_trait]
impl StdinLineWriter for ShellStdinWriter {
    async fn write_line(&self, line: &str) -> Result<(), String> {
        let mut guard = self.child.lock().await;
        let child = guard
            .as_mut()
            .ok_or_else(|| "shell sidecar stdin not available".to_string())?;
        child
            .write(line.as_bytes())
            .map_err(|e| format!("shell sidecar write stdin: {e}"))
    }
}

/// 启动 Tauri shell sidecar 并接入 [`StdioJsonRpcRuntime`] 的 stdout 行分发。
pub fn start_shell_event_loop(
    runtime: Arc<StdioJsonRpcRuntime>,
    shell_writer: Arc<ShellStdinWriter>,
    mut rx: tauri::async_runtime::Receiver<CommandEvent>,
) {
    tokio::spawn(async move {
        let mut line_buf = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) => {
                    line_buf.push_str(&String::from_utf8_lossy(&bytes));
                    while let Some(pos) = line_buf.find('\n') {
                        let line: String = line_buf.drain(..=pos).collect();
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            runtime.dispatch_stdout_line(trimmed).await;
                        }
                    }
                }
                CommandEvent::Stderr(bytes) => {
                    let trimmed = String::from_utf8_lossy(&bytes).trim().to_string();
                    if !trimmed.is_empty() {
                        crate::process_runtime::forward_log_line("channel", &trimmed);
                    }
                }
                CommandEvent::Terminated(_) => {
                    runtime.mark_external_stopped().await;
                    shell_writer.clear().await;
                    runtime.clear_process_state().await;
                    break;
                }
                CommandEvent::Error(message) => {
                    crate::log_warn!("[channel] shell error: {message}");
                }
                _ => {}
            }
        }
        runtime.mark_external_stopped().await;
        shell_writer.clear().await;
        runtime.clear_process_state().await;
    });
}

/// 将键值对写入 Tauri shell 命令环境。
pub fn apply_env_shell(
    mut cmd: tauri_plugin_shell::process::Command,
    extra: &[(impl AsRef<str>, impl AsRef<str>)],
) -> tauri_plugin_shell::process::Command {
    for (key, value) in extra {
        cmd = cmd.env(key.as_ref(), value.as_ref());
    }
    cmd
}
