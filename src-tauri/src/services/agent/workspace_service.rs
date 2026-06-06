//! Workspace-scoped Agent service facade.

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::context::workspace_console;

use super::knowledge::{IngestError, KnowledgeService};
use super::IngestBatchResult;

/// Workspace-scoped facade for knowledge, session index, and channel list operations.
pub struct AgentWorkspaceService {
    workspace: PathBuf,
}

impl AgentWorkspaceService {
    /// Create one workspace service facade for the given root directory.
    ///
    /// # Arguments
    ///
    /// * `workspace` - Agent workspace root directory
    ///
    /// # Returns
    ///
    /// * `AgentWorkspaceService` - Workspace-scoped service facade
    pub fn new(workspace: impl Into<PathBuf>) -> Self {
        Self {
            workspace: workspace.into(),
        }
    }

    /// List persisted sessions for the workspace.
    ///
    /// # Arguments
    ///
    /// * `current_session_id` - Optional active session id to force into the list
    ///
    /// # Returns
    ///
    /// * `Vec<workspace_console::SessionRow>` - Session summary rows
    pub fn list_sessions(
        &self,
        current_session_id: Option<&str>,
    ) -> Result<Vec<workspace_console::SessionRow>, String> {
        workspace_console::list_session_summaries(&self.workspace, current_session_id)
    }

    /// List knowledge markdown files under the workspace.
    ///
    /// # Returns
    ///
    /// * `Vec<workspace_console::KnowledgeFileRow>` - Knowledge file rows
    pub fn list_knowledge_files(&self) -> Result<Vec<workspace_console::KnowledgeFileRow>, String> {
        workspace_console::list_knowledge_files(&self.workspace)
    }

    /// Read one knowledge file by relative path.
    ///
    /// # Arguments
    ///
    /// * `path` - Knowledge-relative file path
    ///
    /// # Returns
    ///
    /// * `String` - Knowledge markdown content
    pub fn read_knowledge_file(&self, path: &str) -> Result<String, String> {
        workspace_console::read_knowledge_file(&self.workspace, path)
    }

    /// Build the workspace knowledge graph.
    ///
    /// # Returns
    ///
    /// * `workspace_console::KnowledgeGraphData` - Graph nodes and links
    pub fn knowledge_graph(&self) -> Result<workspace_console::KnowledgeGraphData, String> {
        workspace_console::build_knowledge_graph(&self.workspace)
    }

    /// Remove one knowledge file by relative path.
    ///
    /// # Arguments
    ///
    /// * `path` - Knowledge-relative file path
    ///
    /// # Returns
    ///
    /// * `()` - Removal result
    pub fn remove_knowledge_file(&self, path: &str) -> Result<(), String> {
        workspace_console::remove_knowledge_file(&self.workspace, path)
    }

    /// Ingest uploaded knowledge files into the workspace.
    ///
    /// # Arguments
    ///
    /// * `files` - Uploaded file tuples `(filename, bytes)`
    /// * `category` - Optional knowledge category
    /// * `config` - Active models config snapshot
    ///
    /// # Returns
    ///
    /// * `IngestBatchResult` - Knowledge ingest result
    pub async fn upload_knowledge_files(
        &self,
        files: Vec<(String, Vec<u8>)>,
        category: Option<&str>,
        config: &models::ModelsConfig,
    ) -> Result<IngestBatchResult, String> {
        let enabled = config.knowledge.unwrap_or(true);
        let svc = KnowledgeService::new(&self.workspace);
        svc.ingest_upload(files, category.unwrap_or("uploads"), true, enabled, config)
            .await
    }

    /// Pick supported knowledge files via native dialog and ingest them.
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri app handle used for native file picking
    /// * `category` - Optional knowledge category
    /// * `config` - Active models config snapshot
    ///
    /// # Returns
    ///
    /// * `IngestBatchResult` - Knowledge ingest result
    pub async fn pick_and_upload_knowledge(
        &self,
        app: &AppHandle,
        category: Option<&str>,
        config: &models::ModelsConfig,
    ) -> Result<IngestBatchResult, String> {
        let maybe_files =
            crate::utils::knowledge_pick::pick_and_read_supported_knowledge_files(app)?;

        let Some(files) = maybe_files else {
            return Ok(IngestBatchResult::default());
        };

        if files.is_empty() {
            return Ok(IngestBatchResult {
                results: Vec::new(),
                errors: vec![IngestError {
                    file: "selection".into(),
                    message: "no files could be read from the chosen paths".into(),
                }],
                count: 0,
                memory_synced: false,
            });
        }

        self.upload_knowledge_files(files, category, config).await
    }

    /// List configured desktop channels from the shared config file.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Shared config file path
    ///
    /// # Returns
    ///
    /// * `Vec<workspace_console::ChannelRow>` - Active channel rows
    pub fn list_channels(
        &self,
        config_path: &Path,
    ) -> Result<Vec<workspace_console::ChannelRow>, String> {
        workspace_console::list_channels_from_config(config_path)
    }
}
