//! In-process agent runtime for Tauri IPC (no Python HTTP).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use agent::SkillEntry;
use agent::{
    get_cancel_registry, Agent, AgentEvent, AgentEventCallback, CancelHandle, McpToolLoader,
    RunStreamOptions,
};
use bridge::{context_from_reply_params, BridgeRuntime};
use models::catalog::provider_configured;
use models::provider_catalog::{
    build_provider_details, find_provider_meta as find_provider_meta_detail,
};
use models::{
    clear_provider_credentials, find_provider_meta, list_providers, set_chat_model,
    update_provider_credentials, ModelsConfig,
};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::context::channel_bridge::ChannelBridge;
use crate::context::workspace_console;
use crate::events::channel_status_changed_all;
use crate::events::names::{AGENT_LOG_STREAM, AGENT_RUN_FINISHED, AGENT_STREAM_CHUNK};
use crate::events::payloads::{
    AgentConsoleState, AgentLogStreamPayload, AgentRunFinished, AgentStreamChunk,
    ChannelStatusChangedPayload, ModelProviderDetail, ModelProviderItem, SkillItem, ToolItem,
};

fn skill_to_item(e: &SkillEntry) -> SkillItem {
    SkillItem {
        name: e.skill.name.clone(),
        description: e.skill.description.clone(),
        enabled: e.enabled,
        source: e.skill.source.clone(),
    }
}

/// Standalone wework / wechat apps connect only from the account UI.
fn should_skip_deferred_channel_autostart() -> bool {
    crate::utils::env::get("DEV_CHANNEL")
        .map(|v| v.trim() == "wework" || v.trim() == "wx")
        .unwrap_or(false)
}

fn deferred_autostart_channels(config_path: &Path) -> Result<Vec<String>, String> {
    let raw = crate::utils::fs::read_to_string(config_path)?;
    let root: serde_json::Value = crate::utils::json::from_str(&raw)?;
    let configured = crate::utils::channel::parse_desktop_channel_types(root.get("channel_type"));
    if let Some(dev_channel) = crate::utils::env::get("DEV_CHANNEL") {
        let trimmed = dev_channel.trim().to_string();
        if trimmed == "wework" || trimmed == "wx" {
            return Ok(Vec::new());
        }
        if !trimmed.is_empty() {
            return Ok(configured
                .into_iter()
                .filter(|name| name == &trimmed)
                .collect());
        }
    }
    Ok(configured)
}

/// Bundled agent config (`src-tauri/resources/config.json` or template). Single source of truth.
fn resolve_bundled_config(app: &AppHandle) -> Result<PathBuf, String> {
    // Dev: `src-tauri/resources/config.json` is gitignored and often not copied to
    // `target/debug/resources/` — read the source tree directly so edits take effect.
    #[cfg(debug_assertions)]
    {
        let source_config = crate::utils::path::crate_path("resources/config.json");
        if source_config.is_file() {
            crate::log_info!(
                "agent config: dev source resources/config.json -> {}",
                source_config.display()
            );
            return Ok(source_config);
        }
    }

    for name in ["config.json", "config-template.json"] {
        let path = app
            .path()
            .resolve(format!("resources/{name}"), BaseDirectory::Resource)
            .map_err(|e| e.to_string())?;
        if path.is_file() {
            crate::log_info!("agent config: resources/{name} -> {}", path.display());
            return Ok(path);
        }
    }
    Err(
        "missing bundled config: place config.json in src-tauri/resources/ (see config-template.json)"
            .into(),
    )
}

/// Writable workspace for agent tools (skills, memory, mcp.json). Config is NOT loaded from here.
fn resolve_workspace_dir(app: &AppHandle) -> Result<PathBuf, String> {
    const ENV_KEY: &str = "SUPPORT_FLOW_WORKSPACE";

    if let Some(path) = crate::utils::env::dir_from_env(ENV_KEY) {
        return Ok(path);
    }

    app.path()
        .app_data_dir()
        .map(|p| p.join("SupportFlow"))
        .map_err(|e| e.to_string())
}

/// Resolve writable workspace + bundled config path.
fn resolve_agent_dirs(app: &AppHandle) -> Result<(PathBuf, PathBuf), String> {
    let config_path = resolve_bundled_config(app)?;
    let workspace = resolve_workspace_dir(app)?;
    crate::utils::fs::create_dir_all(&workspace)?;

    // Mirror into workspace for tools that read ./config.json; source remains resources.
    let mirror = workspace.join("config.json");
    fs::copy(&config_path, &mirror).map_err(|e| format!("sync config to workspace: {e}"))?;

    crate::log_info!(
        "agent workspace: {}, config (resources): {}",
        workspace.display(),
        config_path.display()
    );
    Ok((workspace, config_path))
}

/// Resolve the markitdown_convert.py helper (for Rust knowledge ingest to call out to Python MarkItDown).
/// Prefers bundled resource (listed in tauri.conf), falls back to dev source tree.
fn resolve_markitdown_convert_script(app: &AppHandle) -> Result<PathBuf, String> {
    // Resource (prod bundle + dev if copied to target/.../resources)
    if let Ok(p) = app.path().resolve(
        "channel_agent/scripts/markitdown_convert.py",
        BaseDirectory::Resource,
    ) {
        if p.is_file() {
            return Ok(p);
        }
    }

    // Dev source tree (relative to tauri crate root)
    let dev = crate::utils::path::crate_path("channel_agent/scripts/markitdown_convert.py");
    if dev.is_file() {
        return Ok(dev);
    }

    Err("markitdown_convert.py not located in resources or dev tree".into())
}

fn load_models_config_from_path(path: &Path) -> ModelsConfig {
    if path.is_file() {
        if let Ok(cfg) = ModelsConfig::from_json_file(path) {
            return cfg;
        }
    }
    ModelsConfig {
        bot_type: "deepseek".into(),
        model: Some("deepseek-chat".into()),
        ..Default::default()
    }
}

/// Resolve mirrored workspace config path if present.
#[allow(dead_code)]
pub fn resolve_config_path(workspace: &Path) -> Option<PathBuf> {
    let config = workspace.join("config.json");
    if config.is_file() {
        return Some(config);
    }
    None
}

fn build_bridge_stack(
    workspace: PathBuf,
    config: &ModelsConfig,
    mcp_loader: Arc<McpToolLoader>,
) -> Arc<BridgeRuntime> {
    Arc::new(BridgeRuntime::new(
        workspace,
        Arc::new(config.clone()),
        mcp_loader,
    ))
}

pub struct AgentRuntime {
    app: tauri::AppHandle,
    pub workspace: PathBuf,
    pub config_path: PathBuf,
    config: tokio::sync::RwLock<ModelsConfig>,
    pub mcp_loader: Arc<McpToolLoader>,
    bridge_stack: tokio::sync::RwLock<Arc<BridgeRuntime>>,
    session_id: Mutex<String>,
    log_streaming: tokio::sync::RwLock<bool>,
    channel_sidecar: tokio::sync::Mutex<
        Option<Arc<crate::context::channel_python_sidecar::ChannelPythonSidecar>>,
    >,
    channel_bridge: Arc<ChannelBridge>,
}

#[derive(Clone, Debug)]
pub struct RuntimeMemoryItem {
    pub filename: String,
    pub item_type: String,
    pub size: i32,
    pub updated_at: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeTaskItem {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub next_run_at: Option<String>,
}

fn collect_markdown_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
            continue;
        }
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            out.push(path);
        }
    }
    Ok(())
}

impl AgentRuntime {
    /// Return the owning Tauri app handle for cross-context helpers.
    ///
    /// # Returns
    ///
    /// * `AppHandle` - Cloned app handle
    pub fn app_handle(&self) -> AppHandle {
        self.app.clone()
    }

    /// Initialize runtime with writable workspace, bundled config, and lazy agent state.
    pub fn initialize(app: &AppHandle) -> Result<Self, String> {
        let (workspace, config_path) = resolve_agent_dirs(app)?;
        let config = load_models_config_from_path(&config_path);

        // Wire markitdown script location into env for the parser (upload ingest path).
        // This connects the bundled script (or dev tree) so MarkItDown Python conversion is used
        // before fallbacks, for both Tauri IPC uploads and CLI.
        if let Ok(script_path) = resolve_markitdown_convert_script(app) {
            std::env::set_var("MARKITDOWN_SCRIPT", script_path.to_string_lossy().as_ref());
            crate::log_info!("markitdown script resolved: {}", script_path.display());
        } else {
            crate::log_info!("markitdown script not resolvable via resource (will use dev CARGO fallback if present)");
        }

        let mcp_loader = McpToolLoader::new(workspace.clone());
        mcp_loader.ensure_background_load();
        let session_id = format!("session_{}", uuid::Uuid::new_v4());
        let _ = workspace_console::upsert_session_index(&workspace, &session_id, Some("New Chat"));
        let channel_bridge = Arc::new(ChannelBridge::new());
        let _ = channel_bridge.sync_from_config_file(&config_path);
        let bridge_stack = build_bridge_stack(workspace.clone(), &config, mcp_loader.clone());
        Ok(Self {
            app: app.clone(),
            workspace,
            config_path,
            config: tokio::sync::RwLock::new(config),
            mcp_loader,
            bridge_stack: tokio::sync::RwLock::new(bridge_stack),
            session_id: Mutex::new(session_id),
            log_streaming: tokio::sync::RwLock::new(false),
            channel_sidecar: tokio::sync::Mutex::new(None),
            channel_bridge,
        })
    }

    /// Start Python channel sidecar after the desktop shell is up (non-blocking for Tauri setup).
    pub async fn start_sidecar_deferred(self: Arc<Self>) {
        const DEFAULT_DELAY: Duration = Duration::from_secs(2);
        tokio::time::sleep(DEFAULT_DELAY).await;
        if self.channel_sidecar.lock().await.is_some() {
            return;
        }
        match crate::context::channel_python_sidecar::spawn_sidecar(&self.app, &self.config_path)
            .await
        {
            Ok(sidecar) => {
                sidecar
                    .register_runtime(std::sync::Arc::downgrade(&self))
                    .await;
                *self.channel_sidecar.lock().await = Some(sidecar.clone());
                crate::log_info!("Channel sidecar ready (deferred start)");
                let channels = deferred_autostart_channels(&self.config_path).unwrap_or_default();
                if channels.is_empty() {
                    if should_skip_deferred_channel_autostart() {
                        crate::log_info!(
                            "Channel autostart skipped (DEV_CHANNEL manual-connect preset)"
                        );
                    } else {
                        crate::log_info!("Channel sidecar: no external channels configured");
                    }
                } else {
                    for channel in &channels {
                        if let Err(e) = sidecar.channel_start(channel).await {
                            crate::log_warn!(
                                "Channel autostart start failed for {}: {}",
                                channel,
                                e
                            );
                        }
                    }
                    crate::log_info!("Channel sidecar running: {}", channels.join(", "));
                }
            }
            Err(e) => {
                crate::log_warn!("Channel sidecar deferred start failed: {e}");
            }
        }
    }

    async fn ensure_sidecar(
        self: &Arc<Self>,
    ) -> Result<Arc<crate::context::channel_python_sidecar::ChannelPythonSidecar>, String> {
        if let Some(sidecar) = self.channel_sidecar.lock().await.clone() {
            return Ok(sidecar);
        }
        let sidecar =
            crate::context::channel_python_sidecar::spawn_sidecar(&self.app, &self.config_path)
                .await?;
        sidecar
            .register_runtime(std::sync::Arc::downgrade(self))
            .await;
        *self.channel_sidecar.lock().await = Some(sidecar.clone());
        Ok(sidecar)
    }

    /// Return a running channel sidecar instance, starting it on demand.
    ///
    /// # Returns
    ///
    /// * `Arc<ChannelPythonSidecar>` - Shared sidecar handle ready for runtime RPCs
    pub async fn ensure_channel_sidecar(
        self: &Arc<Self>,
    ) -> Result<Arc<crate::context::channel_python_sidecar::ChannelPythonSidecar>, String> {
        self.ensure_sidecar().await
    }

    /// LLM reply for external channels (Python sidecar calls via `agent.reply` RPC).
    pub async fn channel_reply(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "agent.reply: query required".to_string())?;
        let clear_history = params
            .get("clear_history")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let agent_default = self.config.read().await.agent_enabled();
        let use_agent = params
            .get("agent")
            .and_then(|v| v.as_bool())
            .unwrap_or(agent_default);

        let ctx = context_from_reply_params(params);
        let stack = self.bridge_stack.read().await.clone();
        let reply = stack
            .reply(query, Some(ctx), use_agent, clear_history, None)
            .await;
        Ok(reply.to_json_value())
    }

    /// Channel-agnostic message processing moved into Rust crate (`crates/channel_runtime`).
    pub async fn channel_process(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let ctx_v = params
            .get("context")
            .cloned()
            .ok_or_else(|| "channel.process: missing context".to_string())?;
        let cfg_v = params
            .get("config")
            .cloned()
            .ok_or_else(|| "channel.process: missing config".to_string())?;

        let ctx: channel_runtime::ChannelRuntimeContext =
            serde_json::from_value(ctx_v).map_err(|e| format!("channel.process context: {e}"))?;
        let cfg: channel_runtime::ChannelRuntimeConfig =
            serde_json::from_value(cfg_v).map_err(|e| format!("channel.process config: {e}"))?;
        let result = channel_runtime::process_message(&ctx, &cfg);
        let out = serde_json::to_value(result).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Decorate plain text using Rust channel-runtime rules.
    pub async fn channel_decorate_text(
        &self,
        params: &serde_json::Value,
    ) -> Result<String, String> {
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "channel.decorate_text: missing text".to_string())?;
        let meta_v = params
            .get("meta")
            .cloned()
            .ok_or_else(|| "channel.decorate_text: missing meta".to_string())?;
        let meta: channel_runtime::ChannelRuntimeResult = serde_json::from_value(meta_v)
            .map_err(|e| format!("channel.decorate_text meta: {e}"))?;
        Ok(channel_runtime::decorate_text(text, &meta))
    }

    /// Extract media URLs from text in Rust.
    pub async fn channel_extract_media(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "channel.extract_media: missing text".to_string())?;
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        let items = channel_runtime::extract_media_urls(text, limit);
        serde_json::to_value(items).map_err(|e| e.to_string())
    }

    /// Return a cloned config snapshot for readonly command responses.
    pub async fn config_snapshot(&self) -> ModelsConfig {
        self.config.read().await.clone()
    }

    /// Build aggregated console bootstrap state for frontend.
    ///
    /// # Returns
    ///
    /// * `AgentConsoleState` - Runtime/session/config/provider/tool snapshots for console.
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

    /// Submit one user message and start background stream task.
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri app handle used for event emit.
    /// * `message` - User message content (raw input, will be trimmed).
    ///
    /// # Returns
    ///
    /// * `(String, String)` - `(request_id, session_id)` on accepted run.
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

    /// Refresh skill registry and return latest skill list.
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

    async fn reload_config_from_disk(&self) -> Result<(), String> {
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

    /// Update provider credentials in config and rebuild runtime agent on change.
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

    /// Clear one provider credential set in config and rebuild runtime agent.
    pub async fn clear_provider(&self, provider_id: &str) -> Result<(), String> {
        let meta = find_provider_meta(provider_id)
            .ok_or_else(|| format!("unknown provider: {provider_id}"))?;
        clear_provider_credentials(&self.config_path, meta)?;
        self.reload_config_from_disk().await
    }

    /// Set active provider/model for chat and rebuild runtime agent when changed.
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

    /// Get current runtime session id.
    pub async fn session_id(&self) -> String {
        self.session_id.lock().await.clone()
    }

    /// Create and switch to a new runtime session id.
    pub async fn new_session(&self) -> String {
        let id = format!("session_{}", uuid::Uuid::new_v4());
        *self.session_id.lock().await = id.clone();
        let _ = workspace_console::upsert_session_index(&self.workspace, &id, Some("New Chat"));
        self.bridge_stack
            .write()
            .await
            .agent_bridge
            .clear_all_sessions();
        id
    }

    /// List persisted session summaries for the console sidebar.
    pub async fn list_sessions(&self) -> Result<Vec<workspace_console::SessionRow>, String> {
        let current = self.session_id().await;
        workspace_console::list_session_summaries(&self.workspace, Some(&current))
    }

    /// List knowledge markdown files under workspace.
    pub async fn list_knowledge_files(
        &self,
    ) -> Result<Vec<workspace_console::KnowledgeFileRow>, String> {
        workspace_console::list_knowledge_files(&self.workspace)
    }

    /// Read one knowledge file by relative path.
    pub async fn read_knowledge_file(&self, path: &str) -> Result<String, String> {
        workspace_console::read_knowledge_file(&self.workspace, path)
    }

    /// Build knowledge graph nodes and links.
    pub async fn knowledge_graph(&self) -> Result<workspace_console::KnowledgeGraphData, String> {
        workspace_console::build_knowledge_graph(&self.workspace)
    }

    /// Remove one knowledge file by relative path.
    pub async fn remove_knowledge_file(&self, path: &str) -> Result<(), String> {
        workspace_console::remove_knowledge_file(&self.workspace, path)?;
        Ok(())
    }

    /// Ingest uploads into `knowledge/` (aligned with Python `KnowledgeService.ingest_upload`).
    pub async fn upload_knowledge_files(
        &self,
        files: Vec<(String, Vec<u8>)>,
        category: Option<&str>,
    ) -> Result<agent::IngestBatchResult, String> {
        let config = self.config.read().await.clone();
        let enabled = config.knowledge.unwrap_or(true);
        let svc = agent::knowledge::KnowledgeService::new(&self.workspace);
        svc.ingest_upload(files, category.unwrap_or("uploads"), true, enabled, &config)
            .await
    }

    /// Open the native file picker, read selected files on disk, then ingest into `knowledge/`.
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri app handle (used to open the dialog)
    /// * `category` - optional target category folder under `knowledge/`
    ///
    /// # Returns
    ///
    /// * `IngestBatchResult` - empty success when the user cancels the dialog
    pub async fn pick_and_upload_knowledge(
        &self,
        app: &AppHandle,
        category: Option<&str>,
    ) -> Result<agent::IngestBatchResult, String> {
        let maybe_files =
            crate::utils::knowledge_pick::pick_and_read_supported_knowledge_files(app)?;

        let Some(files) = maybe_files else {
            return Ok(agent::IngestBatchResult::default());
        };

        if files.is_empty() {
            return Ok(agent::IngestBatchResult {
                results: Vec::new(),
                errors: vec![agent::knowledge::IngestError {
                    file: "selection".into(),
                    message: "no files could be read from the chosen paths".into(),
                }],
                count: 0,
                memory_synced: false,
            });
        }

        self.upload_knowledge_files(files, category).await
    }

    /// List configured channels from bundled config.
    pub async fn list_channels(&self) -> Result<Vec<workspace_console::ChannelRow>, String> {
        workspace_console::list_channels_from_config(&self.config_path)
    }

    /// Channel catalog aggregated in Rust using static definitions and runtime status.
    pub async fn channel_python_channels_get(
        self: &Arc<Self>,
    ) -> Result<serde_json::Value, String> {
        crate::context::channel_catalog::build_catalog(&self.app, &self.config_path)
    }

    /// Push channel lifecycle updates from Python sidecar to all Webviews.
    pub fn emit_channel_status_changed(&self, params: &serde_json::Value) {
        let channel = params
            .get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let phase = params
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if channel.is_empty() || phase.is_empty() {
            return;
        }
        let payload = ChannelStatusChangedPayload {
            channel,
            phase,
            message: params
                .get("message")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            user_id: params
                .get("user_id")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            display_name: params
                .get("display_name")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            wait_seconds: params.get("wait_seconds").and_then(|v| v.as_i64()),
            qr_code_url: params
                .get("qr_code_url")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            qr_image: params
                .get("qr_image")
                .and_then(|v| v.as_str())
                .map(str::to_string),
        };
        if let Some(store) = self
            .app
            .try_state::<crate::context::channel_status::ChannelStatusStore>()
        {
            let _ = store.apply(&payload);
        }
        if let Err(e) = channel_status_changed_all(&self.app, &payload) {
            crate::log_warn!("channel status emit failed: {e}");
        }
    }

    /// Channel console APIs handled in Rust using status store plus narrow runtime RPCs.
    pub async fn channel_console_api(
        self: &Arc<Self>,
        path: &str,
        method: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        crate::context::channel_console_api::dispatch(&self.app, self, path, method, &body).await
    }

    pub async fn channel_python_channels_post(
        self: &Arc<Self>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let action = payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let channel = payload
            .get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let config = payload
            .get("config")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();

        let sidecar = self.ensure_sidecar().await?;
        let result = match action.as_str() {
            "save" => {
                let applied = crate::context::channel_runtime::persist_channel_config(
                    &self.config_path,
                    &channel,
                    &config,
                )?;
                self.reload_config_from_disk().await?;
                self.channel_bridge
                    .sync_from_config_file(&self.config_path)?;

                let restarted =
                    crate::context::channel_runtime::should_restart_channel(&channel, &applied);
                if restarted {
                    let _ = sidecar.channel_restart(&channel).await?;
                }

                crate::context::channel_runtime::action_response(
                    self.channel_bridge.active_channels().join(","),
                    restarted,
                    applied,
                )
            }
            "connect" => {
                let (channel_type, applied) = crate::context::channel_runtime::connect_channel(
                    &self.config_path,
                    &channel,
                    &config,
                )?;
                self.reload_config_from_disk().await?;
                self.channel_bridge
                    .sync_from_config_file(&self.config_path)?;
                let _ = sidecar.channel_start(&channel).await?;
                crate::context::channel_runtime::action_response(channel_type, true, applied)
            }
            "disconnect" => {
                let channel_type = crate::context::channel_runtime::disconnect_channel(
                    &self.config_path,
                    &channel,
                )?;
                self.reload_config_from_disk().await?;
                self.channel_bridge
                    .sync_from_config_file(&self.config_path)?;
                let _ = sidecar.channel_stop(&channel).await?;
                crate::context::channel_runtime::action_response(channel_type, true, Vec::new())
            }
            _ => {
                return Err(format!("unknown channel action: {action}"));
            }
        };
        Ok(result)
    }

    /// Enable or disable background log streaming flag.
    pub async fn set_log_streaming(&self, value: bool) {
        *self.log_streaming.write().await = value;
    }

    /// Read background log streaming flag.
    pub async fn log_streaming(&self) -> bool {
        *self.log_streaming.read().await
    }

    /// Resolve latest tauri log source and return `(enabled, source_path)`.
    ///
    /// # Returns
    ///
    /// * `(bool, String)` - Whether log file exists and its absolute path.
    pub async fn logs_status(&self) -> Result<(bool, String), String> {
        let source = resolve_latest_log_path()?;
        Ok((source.exists(), source.display().to_string()))
    }

    /// Start background log tailing and emit `agent/log-stream` events.
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri app handle used for event emitting.
    ///
    /// # Returns
    ///
    /// * `bool` - Whether streaming actually started (false when file missing).
    pub async fn start_log_stream(self: Arc<Self>, app: AppHandle) -> Result<bool, String> {
        let source = resolve_latest_log_path()?;
        if !source.exists() {
            app.emit(
                AGENT_LOG_STREAM,
                AgentLogStreamPayload {
                    payload_type: "error".to_string(),
                    content: None,
                    message: Some("log file not found".to_string()),
                },
            )
            .map_err(|e| e.to_string())?;
            return Ok(false);
        }

        self.set_log_streaming(true).await;
        let app_handle = app.clone();
        let runtime_ref = self.clone();
        let source_path = source.clone();

        tokio::spawn(async move {
            let init = latest_lines_from(&source_path, 500).unwrap_or_default();
            let _ = app_handle.emit(
                AGENT_LOG_STREAM,
                AgentLogStreamPayload {
                    payload_type: "init".to_string(),
                    content: Some(init.clone()),
                    message: None,
                },
            );

            let mut previous_len = init.len();
            let mut last_modified = fs::metadata(&source_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            while runtime_ref.log_streaming().await {
                tokio::time::sleep(Duration::from_millis(900)).await;

                let meta = match fs::metadata(&source_path) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                if modified <= last_modified {
                    continue;
                }
                last_modified = modified;

                let text = match fs::read_to_string(&source_path) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if text.len() < previous_len {
                    previous_len = text.len();
                    let _ = app_handle.emit(
                        AGENT_LOG_STREAM,
                        AgentLogStreamPayload {
                            payload_type: "init".to_string(),
                            content: Some(text),
                            message: None,
                        },
                    );
                    continue;
                }

                let mut slice_start = previous_len.min(text.len());
                while slice_start > 0 && !text.is_char_boundary(slice_start) {
                    slice_start -= 1;
                }
                let delta = text[slice_start..].to_string();
                previous_len = text.len();
                if delta.trim().is_empty() {
                    continue;
                }

                let _ = app_handle.emit(
                    AGENT_LOG_STREAM,
                    AgentLogStreamPayload {
                        payload_type: "line".to_string(),
                        content: Some(delta),
                        message: None,
                    },
                );
            }
        });

        Ok(true)
    }

    /// Stop background log tailing loop.
    pub async fn stop_log_stream(&self) {
        self.set_log_streaming(false).await;
    }

    /// Read latest log lines with optional line limit.
    ///
    /// # Arguments
    ///
    /// * `limit` - Optional number of lines from the end of file.
    ///
    /// # Returns
    ///
    /// * `(String, String)` - Source path and joined log content.
    pub async fn read_logs(&self, limit: Option<i32>) -> Result<(String, String), String> {
        let source = resolve_latest_log_path()?;
        if !source.exists() {
            return Ok((source.display().to_string(), String::new()));
        }

        let raw = crate::utils::fs::read_to_string(&source)?;
        let limit = limit.and_then(|v| usize::try_from(v).ok()).unwrap_or(400);
        let lines: Vec<&str> = raw.lines().collect();
        let start = lines.len().saturating_sub(limit);
        let content = lines[start..].join("\n");
        Ok((source.display().to_string(), content))
    }

    /// 列出工作区下 memory 目录与 MEMORY.md 文件。
    /// List memory markdown files from workspace and map metadata to UI rows.
    pub async fn list_memory_items(&self) -> Result<Vec<RuntimeMemoryItem>, String> {
        let mut files = Vec::new();
        let global = self.workspace.join("MEMORY.md");
        if global.is_file() {
            files.push(global);
        }
        collect_markdown_files(&self.workspace.join("memory"), &mut files)?;

        let mut rows = Vec::with_capacity(files.len());
        for path in files {
            let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or_default();
            let filename = path
                .strip_prefix(&self.workspace)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"));
            let item_type = if filename.contains("/dream") || filename.contains("dream/") {
                "dream".to_string()
            } else if filename.eq_ignore_ascii_case("memory.md") {
                "global".to_string()
            } else {
                "daily".to_string()
            };

            rows.push(RuntimeMemoryItem {
                filename,
                item_type,
                size: i32::try_from(meta.len()).unwrap_or(i32::MAX),
                updated_at: modified.to_string(),
            });
        }
        rows.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(rows)
    }

    /// 读取单个 memory 文件内容（限定在 workspace 下）。
    /// Read one memory markdown file under workspace/memory.
    pub async fn read_memory_item(&self, filename: &str) -> Result<String, String> {
        let rel = filename.replace('\\', "/");
        if rel.contains("..") {
            return Err("invalid memory path".to_string());
        }
        let full = self.workspace.join(rel);
        if !full.starts_with(&self.workspace) {
            return Err("invalid memory path".to_string());
        }
        crate::utils::fs::read_to_string(full)
    }

    /// 读取 scheduler/tasks.json 的任务列表（若不存在返回空数组）。
    /// Parse scheduled tasks from config and map to task rows.
    pub async fn list_task_items(&self) -> Result<Vec<RuntimeTaskItem>, String> {
        let task_path = self.workspace.join("scheduler/tasks.json");
        if !task_path.exists() {
            return Ok(Vec::new());
        }
        let raw = crate::utils::fs::read_to_string(task_path)?;
        let value: serde_json::Value = crate::utils::json::from_str(&raw)?;
        let mut rows = Vec::new();
        let Some(obj) = value.get("tasks").and_then(|v| v.as_object()) else {
            return Ok(rows);
        };

        for (task_id, task) in obj {
            let name = task
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(task_id)
                .to_string();
            let enabled = task
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let next_run_at = task
                .get("next_run_at")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            rows.push(RuntimeTaskItem {
                id: task_id.clone(),
                name,
                enabled,
                next_run_at,
            });
        }
        rows.sort_by(|a, b| b.next_run_at.cmp(&a.next_run_at));
        Ok(rows)
    }

    /// Ensure in-process agent instance is initialized for the current session.
    pub async fn ensure_agent(&self) -> Result<Arc<Agent>, String> {
        let session_id = self.session_id().await;
        let stack = self.bridge_stack.read().await.clone();
        let agent = stack.agent_bridge.ensure_agent(Some(&session_id), "web")?;
        Ok(agent)
    }

    /// Clear conversation history of current in-process agent.
    pub async fn clear_context(&self) -> Result<(), String> {
        let session_id = self.session_id().await;
        let agent = self.ensure_agent().await?;
        agent.clear_history();
        if self
            .config
            .read()
            .await
            .conversation_persistence
            .unwrap_or(true)
        {
            let store = agent::conversation_store_for_workspace(&self.workspace)?;
            store.clear_context(&session_id)?;
        }
        Ok(())
    }

    /// Execute a readonly closure against initialized agent.
    pub async fn with_agent_read<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&Agent) -> R,
    {
        let agent = self.ensure_agent().await?;
        Ok(f(agent.as_ref()))
    }

    /// Execute a closure against initialized agent (`Agent` is behind `Arc`, use interior mutability on tools if needed).
    pub async fn with_agent_write<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&Agent) -> R,
    {
        let agent = self.ensure_agent().await?;
        Ok(f(agent.as_ref()))
    }

    /// Convert low-level agent stream event into frontend stream chunk payload.
    pub fn map_stream_event(request_id: &str, ev: &AgentEvent) -> Option<AgentStreamChunk> {
        let base = |chunk_type: &str, content: Option<String>| AgentStreamChunk {
            request_id: request_id.to_string(),
            chunk_type: chunk_type.to_string(),
            content,
            tool: None,
            arguments: None,
            status: None,
            result: None,
            execution_time: None,
        };

        match ev.event_type.as_str() {
            "reasoning_update" => ev
                .data
                .get("delta")
                .and_then(|v| v.as_str())
                .map(|d| base("reasoning", Some(d.to_string()))),
            "message_update" => ev
                .data
                .get("delta")
                .and_then(|v| v.as_str())
                .map(|d| base("delta", Some(d.to_string()))),
            "tool_execution_start" => Some(AgentStreamChunk {
                request_id: request_id.to_string(),
                chunk_type: "tool_start".into(),
                content: None,
                tool: ev
                    .data
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                arguments: ev.data.get("arguments").cloned(),
                status: None,
                result: None,
                execution_time: None,
            }),
            "tool_execution_end" => Some(AgentStreamChunk {
                request_id: request_id.to_string(),
                chunk_type: "tool_end".into(),
                content: None,
                tool: ev
                    .data
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                arguments: None,
                status: ev
                    .data
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                result: ev.data.get("result").map(|r| {
                    if let Some(s) = r.as_str() {
                        s.to_string()
                    } else {
                        r.to_string()
                    }
                }),
                execution_time: ev.data.get("execution_time").and_then(|v| v.as_f64()),
            }),
            "message_end" => {
                if ev.data.get("cancelled").and_then(|v| v.as_bool()) == Some(true) {
                    Some(base("cancelled", None))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

fn resolve_latest_log_path() -> Result<PathBuf, String> {
    let dirs = directories::ProjectDirs::from("com", "polymerization", "gybte")
        .ok_or_else(|| "could not resolve app log directory".to_string())?;
    let log_root = dirs.data_local_dir().join("logs");
    let date = crate::utils::date::current_date_string();
    Ok(log_root.join(format!("tauri-app.{date}.log")))
}

fn latest_lines_from(path: &PathBuf, limit: usize) -> Result<String, String> {
    let raw = crate::utils::fs::read_to_string(path)?;
    let lines: Vec<&str> = raw.lines().collect();
    let start = lines.len().saturating_sub(limit);
    Ok(lines[start..].join("\n"))
}

/// Run one agent message roundtrip and emit stream lifecycle events.
pub async fn run_agent_message(
    app: AppHandle,
    runtime: Arc<AgentRuntime>,
    request_id: String,
    message: String,
    cancel: CancelHandle,
) {
    if runtime.ensure_agent().await.is_err() {
        let _ = app.emit(
            AGENT_RUN_FINISHED,
            AgentRunFinished {
                request_id,
                error: Some("failed to initialize agent (check config.json / API key)".into()),
                content: None,
            },
        );
        return;
    }

    let request_id_cb = request_id.clone();
    let app_emit = app.clone();
    let on_event: AgentEventCallback = Arc::new(move |ev| {
        if let Some(chunk) = AgentRuntime::map_stream_event(&request_id_cb, &ev) {
            let _ = app_emit.emit(AGENT_STREAM_CHUNK, chunk);
        }
    });

    let run_result = async {
        let agent = runtime.ensure_agent().await?;
        agent
            .run_stream(
                &message,
                RunStreamOptions {
                    on_event: Some(on_event),
                    clear_history: false,
                    cancel: Some(cancel),
                    skill_filter: None,
                },
            )
            .await
            .map_err(|e| e.to_string())
    }
    .await;

    match run_result {
        Ok(content) => {
            let _ = app.emit(
                AGENT_STREAM_CHUNK,
                AgentStreamChunk {
                    request_id: request_id.clone(),
                    chunk_type: "done".into(),
                    content: Some(content.clone()),
                    tool: None,
                    arguments: None,
                    status: None,
                    result: None,
                    execution_time: None,
                },
            );
            let _ = app.emit(
                AGENT_RUN_FINISHED,
                AgentRunFinished {
                    request_id,
                    error: None,
                    content: Some(content),
                },
            );
        }
        Err(e) => {
            let msg = e.to_string();
            let _ = app.emit(
                AGENT_RUN_FINISHED,
                AgentRunFinished {
                    request_id,
                    error: Some(msg),
                    content: None,
                },
            );
        }
    }
}

/// Register cancel handle for one request.
pub fn register_cancel(request_id: &str, session_id: Option<&str>) -> CancelHandle {
    get_cancel_registry().register(request_id, session_id)
}

/// Trigger cancel for one request id.
pub fn cancel_request(request_id: &str) {
    get_cancel_registry().cancel_request(request_id);
}
