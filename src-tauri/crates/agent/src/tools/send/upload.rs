//! Optional cloud upload for `send` tool (`common.cloud_client.copy_send_file`).

use std::path::{Path, PathBuf};

use async_trait::async_trait;

/// Upload local file for channel delivery; returns public URL when available.
#[async_trait]
pub trait SendFileUploader: Send + Sync {
    async fn copy_send_file(&self, absolute_path: &Path, workspace: &Path) -> Option<String>;
}

/// No cloud upload (local / dev).
pub struct NoopSendFileUploader;

#[async_trait]
impl SendFileUploader for NoopSendFileUploader {
    async fn copy_send_file(&self, _absolute_path: &Path, _workspace: &Path) -> Option<String> {
        None
    }
}

pub type SharedSendFileUploader = std::sync::Arc<dyn SendFileUploader>;

pub fn noop_uploader() -> SharedSendFileUploader {
    std::sync::Arc::new(NoopSendFileUploader)
}

/// Callback-based uploader for app integration.
pub struct CallbackSendFileUploader {
    pub f: std::sync::Arc<
        dyn Fn(
                PathBuf,
                PathBuf,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<String>> + Send>>
            + Send
            + Sync,
    >,
}

#[async_trait]
impl SendFileUploader for CallbackSendFileUploader {
    async fn copy_send_file(&self, absolute_path: &Path, workspace: &Path) -> Option<String> {
        (self.f)(absolute_path.to_path_buf(), workspace.to_path_buf()).await
    }
}
