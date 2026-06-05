//! 工作区知识库与会话列表（委托 workspace_console）。

use std::sync::Arc;

use tauri::AppHandle;

use crate::context::workspace_console;

use super::AgentRuntime;

impl AgentRuntime {
    pub async fn list_sessions(&self) -> Result<Vec<workspace_console::SessionRow>, String> {
        let current = self.session_id().await;
        workspace_console::list_session_summaries(&self.workspace, Some(&current))
    }

    pub async fn list_knowledge_files(
        &self,
    ) -> Result<Vec<workspace_console::KnowledgeFileRow>, String> {
        workspace_console::list_knowledge_files(&self.workspace)
    }

    pub async fn read_knowledge_file(&self, path: &str) -> Result<String, String> {
        workspace_console::read_knowledge_file(&self.workspace, path)
    }

    pub async fn knowledge_graph(&self) -> Result<workspace_console::KnowledgeGraphData, String> {
        workspace_console::build_knowledge_graph(&self.workspace)
    }

    pub async fn remove_knowledge_file(&self, path: &str) -> Result<(), String> {
        workspace_console::remove_knowledge_file(&self.workspace, path)?;
        Ok(())
    }

    pub async fn upload_knowledge_files(
        &self,
        files: Vec<(String, Vec<u8>)>,
        category: Option<&str>,
    ) -> Result<crate::agent::IngestBatchResult, String> {
        let config = self.config.read().await.clone();
        let enabled = config.knowledge.unwrap_or(true);
        let svc = crate::agent::knowledge::KnowledgeService::new(&self.workspace);
        svc.ingest_upload(files, category.unwrap_or("uploads"), true, enabled, &config)
            .await
    }

    pub async fn pick_and_upload_knowledge(
        &self,
        app: &AppHandle,
        category: Option<&str>,
    ) -> Result<crate::agent::IngestBatchResult, String> {
        let maybe_files =
            crate::utils::knowledge_pick::pick_and_read_supported_knowledge_files(app)?;

        let Some(files) = maybe_files else {
            return Ok(crate::agent::IngestBatchResult::default());
        };

        if files.is_empty() {
            return Ok(crate::agent::IngestBatchResult {
                results: Vec::new(),
                errors: vec![crate::agent::knowledge::IngestError {
                    file: "selection".into(),
                    message: "no files could be read from the chosen paths".into(),
                }],
                count: 0,
                memory_synced: false,
            });
        }

        self.upload_knowledge_files(files, category).await
    }

    pub async fn list_channels(&self) -> Result<Vec<workspace_console::ChannelRow>, String> {
        workspace_console::list_channels_from_config(&self.config_path)
    }

    pub async fn channel_python_channels_get(
        self: &Arc<Self>,
    ) -> Result<serde_json::Value, String> {
        crate::context::channel::build_catalog(&self.app, &self.config_path)
    }
}
