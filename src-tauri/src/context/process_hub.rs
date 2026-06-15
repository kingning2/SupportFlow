//! 桌面端子进程共享 hub：持有跨 Python/Rust 子进程的共享 context 与各进程 slot。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

use crate::process_runtime::{ProcessSharedContext, ProcessSlot};
use tauri::AppHandle;

use crate::context::agent_runtime::AgentRuntime;

/// 所有子进程共享的桌面 context + 按进程类型划分的懒加载 slot。
pub struct ProcessHub {
    pub shared: ProcessSharedContext,
    channel: ProcessSlot<crate::python::ChannelPythonSidecar>,
}

impl ProcessHub {
    /// 从 Agent 工作区与配置路径构建共享 hub。
    pub fn new(workspace: PathBuf, config_path: &Path) -> Self {
        let mut shared = ProcessSharedContext::new(workspace);
        shared.env.insert(
            "CHANNEL_CONFIG_PATH".into(),
            config_path.display().to_string(),
        );
        Self {
            shared,
            channel: ProcessSlot::new(),
        }
    }

    pub fn channel_slot(&self) -> &ProcessSlot<crate::python::ChannelPythonSidecar> {
        &self.channel
    }

    /// 懒加载渠道 sidecar，并在首次启动时注册 [`AgentRuntime`] 弱引用。
    pub async fn ensure_channel(
        &self,
        app: &AppHandle,
        host: Weak<AgentRuntime>,
    ) -> Result<Arc<crate::python::ChannelPythonSidecar>, String> {
        self.channel
            .ensure(|| async {
                let sidecar = crate::python::spawn_sidecar(app, &self.shared).await?;
                sidecar.register_runtime(host).await;
                Ok(sidecar)
            })
            .await
    }
}
