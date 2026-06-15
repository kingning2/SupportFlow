//! 外部 stdin 写入契约。

use async_trait::async_trait;

/// 外部 stdin 写入器（Tauri shell sidecar 等非 tokio stdin 场景）。
#[async_trait]
pub trait StdinLineWriter: Send + Sync {
    async fn write_line(&self, line: &str) -> Result<(), String>;
}
