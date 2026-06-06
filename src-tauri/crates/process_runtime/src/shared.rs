//! 所有子进程共享的运行时上下文。

use std::collections::HashMap;
use std::path::PathBuf;

/// 跨进程、跨语言（Python / Rust exe）共享的桌面运行时上下文。
///
/// 由 `context::ProcessHub` 在应用启动时构建；子进程通过环境变量或 spec 注入读取。
#[derive(Clone, Default)]
pub struct ProcessSharedContext {
    pub workspace: PathBuf,
    pub env: HashMap<String, String>,
}

impl ProcessSharedContext {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            workspace,
            env: HashMap::new(),
        }
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn merge_env(&self, out: &mut Vec<(String, String)>) {
        for (k, v) in &self.env {
            out.push((k.clone(), v.clone()));
        }
    }
}
