//! Shared workspace config for file tools.

use std::path::{Path, PathBuf};

use crate::services::agent::tools::utils::path::resolve_path;

#[derive(Debug, Clone)]
pub struct WorkspaceToolConfig {
    pub cwd: PathBuf,
}

impl WorkspaceToolConfig {
    pub fn resolve(&self, path: &str) -> PathBuf {
        resolve_path(&self.cwd, path)
    }

    pub fn display_cwd(&self) -> &Path {
        &self.cwd
    }
}
