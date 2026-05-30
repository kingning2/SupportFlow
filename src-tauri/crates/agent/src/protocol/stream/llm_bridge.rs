//! `bridge/agent_bridge.py` — `AgentLLMModel` adapter over `models` bots.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Stream;
use models::{CallWithToolsRequest, LlmResult, OpenAICompatibleBot};
use serde_json::{json, Map, Value};
use tracing::{debug, error};

use crate::protocol::LlmRequest;

/// Runtime toggles mirrored from `config.json` / `agent_bridge` kwargs.
#[derive(Debug, Clone, Default)]
pub struct LlmBridgeConfig {
    pub model: String,
    pub enable_thinking: bool,
    pub reasoning_effort: Option<String>,
    pub channel_type: String,
    pub session_id: Option<String>,
}

/// Stream of OpenAI-shaped chunks consumed by `AgentStreamExecutor`.
pub type LlmChunkStream = Pin<Box<dyn Stream<Item = Value> + Send>>;

#[derive(Debug, thiserror::Error)]
pub enum LlmBridgeError {
    #[error("LLM call failed: {0}")]
    Message(String),
}

/// Abstraction over Python `LLMModel` for agent streaming.
#[async_trait]
pub trait LlmModel: Send + Sync {
    fn model_name(&self) -> &str;
    fn channel_type(&self) -> &str;
    async fn call_stream(&self, request: &LlmRequest) -> Result<LlmChunkStream, LlmBridgeError>;
}

/// Wraps `models::OpenAICompatibleBot` (`AgentLLMModel` in Python).
pub struct BotLlmModel {
    bot: Arc<dyn OpenAICompatibleBot>,
    bridge: LlmBridgeConfig,
}

impl BotLlmModel {
    pub fn new(bot: Arc<dyn OpenAICompatibleBot>, bridge: LlmBridgeConfig) -> Self {
        Self { bot, bridge }
    }

    pub fn with_bridge(bot: Arc<dyn OpenAICompatibleBot>, bridge: LlmBridgeConfig) -> Self {
        Self { bot, bridge }
    }

    fn build_call_request(&self, request: &LlmRequest) -> CallWithToolsRequest {
        let mut extra = Map::new();
        if self.bridge.enable_thinking {
            extra.insert("thinking".into(), json!({"type": "enabled"}));
            if let Some(effort) = &self.bridge.reasoning_effort {
                if effort == "high" || effort == "max" {
                    extra.insert("reasoning_effort".into(), json!(effort));
                }
            }
        } else {
            extra.insert("thinking".into(), json!({"type": "disabled"}));
        }

        CallWithToolsRequest {
            messages: request.messages.clone(),
            tools: request.tools.clone(),
            stream: true,
            model: request
                .model
                .clone()
                .or_else(|| Some(self.bridge.model.clone()))
                .filter(|s| !s.is_empty()),
            max_tokens: request.max_tokens,
            temperature: Some(request.temperature),
            system: request.system.clone(),
            extra,
            ..Default::default()
        }
    }
}

#[async_trait]
impl LlmModel for BotLlmModel {
    fn model_name(&self) -> &str {
        &self.bridge.model
    }

    fn channel_type(&self) -> &str {
        &self.bridge.channel_type
    }

    async fn call_stream(&self, request: &LlmRequest) -> Result<LlmChunkStream, LlmBridgeError> {
        let req = self.build_call_request(request);
        debug!(
            model = %self.bridge.model,
            tools = req.tools.as_ref().map(|t| t.len()).unwrap_or(0),
            "BotLlmModel call_stream"
        );
        match self.bot.call_with_tools(req).await {
            Ok(LlmResult::Stream(stream)) => Ok(stream),
            Ok(LlmResult::Complete(body)) => {
                let mut chunks: Vec<Value> = Vec::new();
                if body.get("error") == Some(&Value::Bool(true)) {
                    chunks.push(body);
                } else {
                    chunks.push(body);
                }
                Ok(Box::pin(futures_util::stream::iter(chunks)))
            }
            Err(e) => {
                error!(error = %e, "call_with_tools failed");
                Err(LlmBridgeError::Message(e.message))
            }
        }
    }
}
