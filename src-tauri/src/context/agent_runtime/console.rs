//! 控制台状态、消息发送与模型配置更新。

use std::fs;
use std::sync::Arc;

use models::catalog::provider_configured;
use models::provider_catalog::{
    build_provider_details, find_provider_meta as find_provider_meta_detail,
};
use models::{
    clear_provider_credentials, find_provider_meta, list_providers, set_chat_model,
    update_provider_credentials, ModelsConfig,
};
use tauri::AppHandle;

use crate::context::workspace_console;
use crate::events::payloads::{
    AgentConsoleState, ModelProviderDetail, ModelProviderItem, SkillItem, ToolItem,
};

use super::helpers::{build_bridge_stack, load_models_config_from_path, skill_to_item};
use super::stream::{register_cancel, run_agent_message};
use super::AgentRuntime;

impl AgentRuntime {
    pub async fn config_snapshot(&self) -> ModelsConfig {
        self.config.read().await.clone()
    }

    pub async fn console_state(&self) -> Result<AgentConsoleState, String> {
        self.ensure_agent().await?;
        let session_id = self.session_id().await;
        let workspace = self.workspace.display().to_string();
        let config = self.config_snapshot().await;
        let config_model_fallback = config.model_or("unknown");

        let (model_name, tools, skills) = self
            .with_agent_read(|agent| {
                let tools: Vec<ToolItem> = agent
                    .tools
                    .iter()
                    .map(|t| ToolItem {
                        name: t.name().to_string(),
                        description: t.description().to_string(),
                        is_mcp: t.is_mcp(),
                    })
                    .collect();
                let skills: Vec<SkillItem> = agent
                    .list_skills()
                    .into_iter()
                    .map(|e| skill_to_item(&e))
                    .collect();
                let model_name = agent
                    .model
                    .as_ref()
                    .map(|m| m.model_name().to_string())
                    .unwrap_or_else(|| config_model_fallback.clone());
                (model_name, tools, skills)
            })
            .await?;

        let mcp_status = self.mcp_loader.list_mcp_status();
        let providers: Vec<ModelProviderItem> = list_providers(&config)
            .into_iter()
            .map(|p| ModelProviderItem {
                id: p.id,
                configured: p.configured,
                is_active: p.is_active,
            })
            .collect();
        let provider_details: Vec<ModelProviderDetail> = build_provider_details(&config)
            .into_iter()
            .map(|d| {
                let editable = find_provider_meta_detail(&d.id).is_some();
                ModelProviderDetail {
                    id: d.id,
                    configured: d.configured,
                    is_active: d.is_active,
                    api_base: d.api_base,
                    api_base_default: d.api_base_default,
                    has_api_base: d.has_api_base,
                    api_key_masked: d.api_key_masked,
                    models: d.models,
                    bot_type_value: d.bot_type_value,
                    editable,
                }
            })
            .collect();

        Ok(AgentConsoleState {
            session_id,
            workspace_dir: workspace,
            model_name,
            bot_type: config.bot_type.clone(),
            providers,
            provider_details,
            tools,
            skills,
            mcp_status,
            config_path: Some(self.config_path.display().to_string()),
            temperature: config.temperature,
            top_p: config.top_p,
            request_timeout: config
                .request_timeout
                .and_then(|secs| u32::try_from(secs).ok()),
        })
    }

    pub async fn send_message(
        self: Arc<Self>,
        app: AppHandle,
        message: String,
    ) -> Result<(String, String), String> {
        let message = message.trim().to_string();
        if message.is_empty() {
            return Err("message is empty".into());
        }

        let title_hint = if message.chars().count() > 30 {
            format!("{}...", message.chars().take(30).collect::<String>())
        } else {
            message.clone()
        };
        let session_id_for_index = self.session_id().await;
        let workspace = self.workspace.clone();
        let _ = workspace_console::upsert_session_index(
            &workspace,
            &session_id_for_index,
            Some(&title_hint),
        );

        self.ensure_agent().await?;
        let session_id = self.session_id().await;

        let config = self.config_snapshot().await;
        let bot_type = config.bot_type().map_err(|e| e.to_string())?;
        if !provider_configured(bot_type, &config) {
            return Err(format!(
                "API key not configured for bot_type \"{}\". Configure it on the Models page or in config.json.",
                config.bot_type
            ));
        }

        let request_id = uuid::Uuid::new_v4().to_string();
        let cancel = register_cancel(&request_id, Some(&session_id));
        let rt = self.clone();
        let app2 = app.clone();
        let rid = request_id.clone();
        tokio::spawn(async move {
            run_agent_message(app2, rt, rid, message, cancel).await;
        });

        Ok((request_id, session_id))
    }

    pub async fn refresh_skills(&self) -> Result<Vec<SkillItem>, String> {
        self.with_agent_write(|agent| {
            agent.refresh_skills();
            agent
                .list_skills()
                .into_iter()
                .map(|e| skill_to_item(&e))
                .collect::<Vec<_>>()
        })
        .await
    }

    pub(crate) async fn reload_config_from_disk(&self) -> Result<(), String> {
        let fresh = load_models_config_from_path(&self.config_path);
        *self.config.write().await = fresh;
        let mirror = self.workspace.join("config.json");
        fs::copy(&self.config_path, &mirror)
            .map_err(|e| format!("sync config to workspace: {e}"))?;
        let fresh = self.config.read().await.clone();
        *self.bridge_stack.write().await =
            build_bridge_stack(self.workspace.clone(), &fresh, self.mcp_loader.clone());
        Ok(())
    }

    pub async fn update_provider(
        &self,
        provider_id: &str,
        api_key: Option<&str>,
        api_base: Option<&str>,
        api_base_set: bool,
    ) -> Result<(), String> {
        let meta = find_provider_meta(provider_id)
            .ok_or_else(|| format!("unknown provider: {provider_id}"))?;
        let changed =
            update_provider_credentials(&self.config_path, meta, api_key, api_base, api_base_set)?;
        if changed {
            self.reload_config_from_disk().await?;
        }
        Ok(())
    }

    pub async fn clear_provider(&self, provider_id: &str) -> Result<(), String> {
        let meta = find_provider_meta(provider_id)
            .ok_or_else(|| format!("unknown provider: {provider_id}"))?;
        clear_provider_credentials(&self.config_path, meta)?;
        self.reload_config_from_disk().await
    }

    pub async fn set_active_chat(
        &self,
        provider_id: Option<&str>,
        model: Option<&str>,
    ) -> Result<(), String> {
        let changed = set_chat_model(&self.config_path, provider_id, model)?;
        if changed {
            self.reload_config_from_disk().await?;
        }
        Ok(())
    }
}
