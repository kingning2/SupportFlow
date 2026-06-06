//! 子进程命令描述（语言无关：Rust exe、Python 解释器、任意二进制）。

use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::shared::ProcessSharedContext;
use crate::env::piped_stdio;

/// 启动任意子进程所需的命令规格。
#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub name: &'static str,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Vec<(String, String)>,
    pub piped: bool,
}

impl CommandSpec {
    pub fn binary(name: &'static str, program: impl Into<PathBuf>) -> Self {
        Self {
            name,
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: Vec::new(),
            piped: false,
        }
    }

    pub fn with_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn piped(mut self) -> Self {
        self.piped = true;
        self
    }

    pub fn merged_env(&self, shared: &ProcessSharedContext) -> Vec<(String, String)> {
        let mut out = Vec::new();
        shared.merge_env(&mut out);
        out.extend(self.env.clone());
        out
    }

    pub fn into_tokio_command(&self, shared: &ProcessSharedContext) -> Command {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args);
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        for (k, v) in self.merged_env(shared) {
            cmd.env(k, v);
        }
        if self.piped {
            piped_stdio(&mut cmd);
        }
        cmd
    }

    pub fn into_std_command(&self, shared: &ProcessSharedContext) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.program);
        cmd.args(&self.args);
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        for (k, v) in self.merged_env(shared) {
            cmd.env(k, v);
        }
        cmd
    }

    pub fn validate_program(&self) -> Result<(), String> {
        if self.program.is_file() {
            return Ok(());
        }
        Err(format!(
            "process {}: program not found: {}",
            self.name,
            self.program.display()
        ))
    }
}

/// 从环境变量解析可执行文件覆盖路径。
pub fn resolve_exe_from_env(env_key: &str) -> Result<Option<PathBuf>, String> {
    let Some(raw) = std::env::var(env_key).ok() else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let path = PathBuf::from(trimmed);
    if path.is_file() {
        return Ok(Some(path));
    }
    Err(format!(
        "{env_key} points to missing file: {}",
        path.display()
    ))
}

/// 拼接 `binaries/{name}-{target}{EXE_SUFFIX}` 相对路径并检查存在性。
pub fn binary_in_dir(binaries_dir: &Path, stem: &str, target_triple: &str) -> PathBuf {
    binaries_dir.join(format!(
        "{stem}-{target_triple}{}",
        std::env::consts::EXE_SUFFIX
    ))
}
