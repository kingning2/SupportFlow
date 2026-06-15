//! Workspace-scoped Agent service facade.

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use super::data::{self, ChannelRow, KnowledgeFileRow, KnowledgeGraphData, SessionRow};
use crate::services::agent::knowledge::{IngestError, KnowledgeService};
use crate::services::agent::IngestBatchResult;

/// Workspace-scoped facade for knowledge, session index, and channel list operations.
pub struct AgentWorkspaceService {
    workspace: PathBuf,
}

impl AgentWorkspaceService {
    pub fn new(workspace: impl Into<PathBuf>) -> Self {
        Self {
            workspace: workspace.into(),
        }
    }

    pub fn list_sessions(
        &self,
        current_session_id: Option<&str>,
    ) -> Result<Vec<SessionRow>, String> {
        data::list_session_summaries(&self.workspace, current_session_id)
    }

    pub fn list_knowledge_files(&self) -> Result<Vec<KnowledgeFileRow>, String> {
        data::list_knowledge_files(&self.workspace)
    }

    pub fn read_knowledge_file(&self, path: &str) -> Result<String, String> {
        data::read_knowledge_file(&self.workspace, path)
    }

    pub fn knowledge_graph(&self) -> Result<KnowledgeGraphData, String> {
        data::build_knowledge_graph(&self.workspace)
    }

    pub fn remove_knowledge_file(&self, path: &str) -> Result<(), String> {
        data::remove_knowledge_file(&self.workspace, path)
    }

    pub async fn upload_knowledge_files(
        &self,
        files: Vec<(String, Vec<u8>)>,
        category: Option<&str>,
        config: &crate::config::ModelsConfig,
    ) -> Result<IngestBatchResult, String> {
        let enabled = config.knowledge.unwrap_or(true);
        let svc = KnowledgeService::new(&self.workspace);
        svc.ingest_upload(files, category.unwrap_or("uploads"), true, enabled, config)
            .await
    }

    pub async fn pick_and_upload_knowledge(
        &self,
        app: &AppHandle,
        category: Option<&str>,
        config: &crate::config::ModelsConfig,
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

    pub fn list_channels(&self, config_path: &Path) -> Result<Vec<ChannelRow>, String> {
        data::list_channels_from_config(config_path)
    }
}
