//! 工作区知识库与会话列表（委托 workspace_console）。

use std::sync::Arc;

use tauri::AppHandle;

use crate::context::workspace_console;
use crate::services::agent::AgentWorkspaceService;

use super::AgentRuntime;

impl AgentRuntime {
    pub async fn list_sessions(&self) -> Result<Vec<workspace_console::SessionRow>, String> {
        let current = self.session_id().await;
        AgentWorkspaceService::new(&self.workspace).list_sessions(Some(&current))
    }

    pub async fn list_knowledge_files(
        &self,
    ) -> Result<Vec<workspace_console::KnowledgeFileRow>, String> {
        AgentWorkspaceService::new(&self.workspace).list_knowledge_files()
    }

    pub async fn read_knowledge_file(&self, path: &str) -> Result<String, String> {
        AgentWorkspaceService::new(&self.workspace).read_knowledge_file(path)
    }

    pub async fn knowledge_graph(&self) -> Result<workspace_console::KnowledgeGraphData, String> {
        AgentWorkspaceService::new(&self.workspace).knowledge_graph()
    }

    pub async fn remove_knowledge_file(&self, path: &str) -> Result<(), String> {
        AgentWorkspaceService::new(&self.workspace).remove_knowledge_file(path)
    }

    pub async fn upload_knowledge_files(
        &self,
        files: Vec<(String, Vec<u8>)>,
        category: Option<&str>,
    ) -> Result<crate::services::agent::IngestBatchResult, String> {
        let config = self.config.read().await.clone();
        AgentWorkspaceService::new(&self.workspace)
            .upload_knowledge_files(files, category, &config)
            .await
    }

    pub async fn pick_and_upload_knowledge(
        &self,
        app: &AppHandle,
        category: Option<&str>,
    ) -> Result<crate::services::agent::IngestBatchResult, String> {
        let config = self.config.read().await.clone();
        AgentWorkspaceService::new(&self.workspace)
            .pick_and_upload_knowledge(app, category, &config)
            .await
    }

    pub async fn list_channels(&self) -> Result<Vec<workspace_console::ChannelRow>, String> {
        AgentWorkspaceService::new(&self.workspace).list_channels(&self.config_path)
    }

    pub async fn channel_python_channels_get(
        self: &Arc<Self>,
    ) -> Result<serde_json::Value, String> {
        crate::context::channel::build_catalog(&self.app, &self.config_path)
    }
}
