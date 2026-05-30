//! `models/deepseek/deepseek_bot.py`

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Map, Value};
use tokio::time::sleep;
use tracing::{debug, error, info};

use crate::bot::{Bot, BotError};
use crate::bridge::{Context, ContextType, Reply};
use crate::channel_reply::{reply_from_text_result, try_admin_commands, ReplyTextResult};
use crate::config::ModelsConfig;
use crate::const_::BotType;
use crate::openai::OpenAiHttpClient;
use crate::openai::OpenAiHttpError;
use crate::openai_compatible::{ApiConfig, CallWithToolsRequest, LlmResult, OpenAICompatibleBot};
use crate::provider::{api_config, http_client, session_manager, SharedConfig};
use crate::session::SessionClass;

const DEFAULT_API_BASE: &str = "https://api.deepseek.com/v1";

/// `DeepSeekBot` — mirrors Python class fields.
#[derive(Debug)]
pub struct DeepSeekBot {
    pub(crate) config: SharedConfig,
    pub sessions: crate::session_manager::SessionManager,
    pub(crate) client: OpenAiHttpClient,
    pub(crate) args: ChatArgs,
}

#[derive(Debug, Clone)]
pub struct ChatArgs {
    pub model: String,
    pub temperature: f32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

impl DeepSeekBot {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
        let model = config.model_or(BotType::DEEPSEEK_V4_FLASH);
        let api_key = config
            .deepseek_api_key
            .clone()
            .or_else(|| config.open_ai_api_key.clone())
            .unwrap_or_default();
        let api_base = config
            .deepseek_api_base
            .clone()
            .or_else(|| config.open_ai_api_base.clone())
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string());

        let args = ChatArgs {
            model: model.clone(),
            temperature: config.temperature_or(0.7),
            top_p: config.top_p_or(1.0),
            frequency_penalty: config.frequency_penalty_or(0.0),
            presence_penalty: config.presence_penalty_or(0.0),
        };

        Self {
            sessions: session_manager(&config, SessionClass::StandardChatBlocks, &model),
            client: http_client(&config, Some(api_key), Some(api_base)),
            config,
            args,
        }
    }

    pub fn model_supports_thinking(model_name: &str) -> bool {
        model_name.to_lowercase().starts_with("deepseek-v4")
    }

    pub fn is_reasoner_model(model_name: &str) -> bool {
        !model_name.is_empty() && model_name.to_lowercase().contains("reasoner")
    }

    pub(crate) fn api_key(&self) -> String {
        self.config
            .deepseek_api_key
            .clone()
            .or_else(|| self.config.open_ai_api_key.clone())
            .unwrap_or_default()
    }

    pub(crate) fn api_base(&self) -> String {
        self.config
            .deepseek_api_base
            .clone()
            .or_else(|| self.config.open_ai_api_base.clone())
            .map(|s| s.trim_end_matches('/').to_string())
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string())
    }

    /// `reply_text` — non-agent chat completion with retry.
    pub async fn reply_text(
        &self,
        messages: &[Value],
        args: Option<&ChatArgs>,
        retry_count: u32,
    ) -> ReplyTextResult {
        let args = args.unwrap_or(&self.args);
        let mut body = Map::new();
        body.insert("model".into(), json!(args.model));
        body.insert("messages".into(), Value::Array(messages.to_vec()));
        body.insert("temperature".into(), json!(args.temperature));
        body.insert("top_p".into(), json!(args.top_p));
        body.insert("frequency_penalty".into(), json!(args.frequency_penalty));
        body.insert("presence_penalty".into(), json!(args.presence_penalty));

        let model_name = args.model.as_str();
        if Self::model_supports_thinking(model_name) || Self::is_reasoner_model(model_name) {
            for k in [
                "temperature",
                "top_p",
                "presence_penalty",
                "frequency_penalty",
            ] {
                body.remove(k);
            }
        }

        let api_key = self.api_key();
        let api_base = self.api_base();

        match self
            .client
            .chat_completions(
                body,
                if api_key.is_empty() {
                    None
                } else {
                    Some(api_key.as_str())
                },
                Some(api_base.as_str()),
                Some(180),
            )
            .await
        {
            Ok(response) => {
                let usage = response.get("usage").cloned().unwrap_or(Value::Null);
                let content = response
                    .get("choices")
                    .and_then(|c| c.get(0))
                    .and_then(|c| c.get("message"))
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                ReplyTextResult {
                    total_tokens: usage
                        .get("total_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    completion_tokens: usage
                        .get("completion_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    content,
                }
            }
            Err(e) => {
                error!(
                    status = e.status_code,
                    msg = %e.message,
                    "deepseek chat failed"
                );
                let mut result = ReplyTextResult {
                    total_tokens: 0,
                    completion_tokens: 0,
                    content: "提问太快啦，请休息一下再问我吧".to_string(),
                };
                let mut need_retry = e.status_code == 0 && retry_count < 2;
                if e.status_code >= 500 {
                    need_retry = retry_count < 2;
                } else if e.status_code == 401 {
                    result.content = "授权失败，请检查API Key是否正确".to_string();
                } else if e.status_code == 429 {
                    result.content = "请求过于频繁，请稍后再试".to_string();
                    need_retry = retry_count < 2;
                }
                if need_retry {
                    sleep(Duration::from_secs(3)).await;
                    return Box::pin(self.reply_text(messages, Some(args), retry_count + 1)).await;
                }
                result
            }
        }
    }
}

#[async_trait]
impl Bot for DeepSeekBot {
    async fn reply(&self, query: &str, context: Option<&Context>) -> Result<Reply, BotError> {
        let ctx = context.ok_or_else(|| BotError::Message("context is required".into()))?;
        if ctx.get_type() != Some(ContextType::Text) {
            return Ok(Reply::error(format!(
                "Bot不支持处理{:?}类型的消息",
                ctx.get_type()
            )));
        }

        info!(query, "[DEEPSEEK] query");

        let session_id = ctx
            .session_id()
            .ok_or_else(|| BotError::Message("session_id required".into()))?;

        if let Some(admin) = try_admin_commands(query, &self.config, &self.sessions, session_id) {
            return Ok(admin);
        }

        let session = self.sessions.session_query(query, session_id);
        debug!(messages = ?session.messages(), "[DEEPSEEK] session query");

        let reply_content = self
            .reply_text(session.messages(), Some(&self.args), 0)
            .await;
        debug!(
            session_id,
            completion_tokens = reply_content.completion_tokens,
            content = %reply_content.content,
            "[DEEPSEEK] reply_text"
        );

        if reply_content.completion_tokens > 0 {
            self.sessions.session_reply(
                &reply_content.content,
                session_id,
                Some(reply_content.total_tokens),
            );
        }

        Ok(reply_from_text_result(&reply_content))
    }
}

#[async_trait]
impl OpenAICompatibleBot for DeepSeekBot {
    fn get_api_config(&self) -> ApiConfig {
        api_config(
            self.api_key(),
            self.api_base(),
            self.config.model_or(BotType::DEEPSEEK_V4_FLASH),
            &self.config,
        )
    }

    fn http_client(&self) -> &OpenAiHttpClient {
        &self.client
    }

    async fn call_with_tools(
        &self,
        req: CallWithToolsRequest,
    ) -> Result<LlmResult, OpenAiHttpError> {
        self.call_with_tools_deepseek(req).await
    }
}
