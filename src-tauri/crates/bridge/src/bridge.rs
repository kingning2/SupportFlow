//! `bridge/bridge.py` — bot routing, voice/translate stubs, agent delegation.

use std::collections::HashMap;
use std::sync::Arc;

use models::openai_compatible::{CallWithToolsRequest, LlmResult, OpenAICompatibleBot};
use models::{create_bot, BotType, Context, ModelsConfig, Reply};
use serde_json::json;
use tracing::info;

use crate::agent_bridge::AgentBridge;
use crate::bot_router::{auto_pick_voice_to_text, resolve_bot_type};

/// SupportFlow Agent `Bridge` (without Python voice/translate backends).
pub struct Bridge {
    pub config: Arc<ModelsConfig>,
    chat_bot_type: BotType,
    voice_to_text_provider: String,
    text_to_voice_provider: String,
    bots: std::sync::Mutex<HashMap<String, Arc<dyn OpenAICompatibleBot>>>,
    agent_bridge: std::sync::Mutex<Option<Arc<AgentBridge>>>,
}

impl Bridge {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
        let chat_bot_type = resolve_bot_type(&config).unwrap_or(BotType::Deepseek);
        let mut voice_to_text = config.voice_to_text.clone().unwrap_or_default();
        if voice_to_text.is_empty() {
            voice_to_text = auto_pick_voice_to_text(&config).to_string();
        }
        let mut text_to_voice = config
            .text_to_voice
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "google".to_string());

        if config.use_linkai.unwrap_or(false) && config.has_linkai_key() {
            let v2t = config.voice_to_text.as_deref().unwrap_or("");
            if v2t.is_empty() || v2t == "openai" {
                voice_to_text = "linkai".to_string();
            }
            let t2v = config.text_to_voice.as_deref().unwrap_or("");
            if t2v.is_empty() || matches!(t2v, "openai" | "tts-1" | "tts-1-hd") {
                text_to_voice = "linkai".to_string();
            }
        }

        Self {
            config,
            chat_bot_type,
            voice_to_text_provider: voice_to_text,
            text_to_voice_provider: text_to_voice,
            bots: std::sync::Mutex::new(HashMap::new()),
            agent_bridge: std::sync::Mutex::new(None),
        }
    }

    pub fn refresh_voice(&self) {
        let voice_to_text = self
            .config
            .voice_to_text
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| auto_pick_voice_to_text(&self.config).to_string());
        let text_to_voice = self
            .config
            .text_to_voice
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "google".to_string());
        let mut bots = self.bots.lock().expect("bots");
        bots.remove("voice_to_text");
        bots.remove("text_to_voice");
        info!(
            "[Bridge] voice refreshed: voice_to_text={}, text_to_voice={}",
            voice_to_text, text_to_voice
        );
    }

    pub fn reset_bot(&mut self) {
        *self = Self::new(self.config.clone());
    }

    fn get_chat_bot(&self) -> Result<Arc<dyn OpenAICompatibleBot>, String> {
        let mut bots = self.bots.lock().expect("bots");
        if let Some(b) = bots.get("chat") {
            return Ok(b.clone());
        }
        let bot = create_bot(self.chat_bot_type, self.config.clone())?;
        bots.insert("chat".to_string(), bot.clone());
        Ok(bot)
    }

    /// Non-agent chat completion (`Bridge.fetch_reply_content`).
    pub async fn fetch_reply_content(&self, query: &str, _context: Option<&Context>) -> Reply {
        match self.get_chat_bot() {
            Ok(bot) => match simple_bot_reply(bot.as_ref(), query).await {
                Ok(reply) => reply,
                Err(e) => Reply::error(e),
            },
            Err(e) => Reply::error(e),
        }
    }

    pub async fn fetch_voice_to_text(&self, _voice_file: &str) -> Reply {
        let _ = &self.voice_to_text_provider;
        Reply::error(
            "voice_to_text: not implemented in Rust (configure provider in a future release)",
        )
    }

    pub async fn fetch_text_to_voice(&self, _text: &str) -> Reply {
        let _ = &self.text_to_voice_provider;
        Reply::error(
            "text_to_voice: not implemented in Rust (configure provider in a future release)",
        )
    }

    pub async fn fetch_translate(&self, text: &str, _from_lang: &str, _to_lang: &str) -> Reply {
        Reply::text(text)
    }

    pub fn attach_agent_bridge(&self, ab: Arc<AgentBridge>) {
        *self.agent_bridge.lock().expect("agent_bridge") = Some(ab);
    }

    pub fn agent_bridge(&self) -> Option<Arc<AgentBridge>> {
        self.agent_bridge.lock().expect("agent_bridge").clone()
    }
}

async fn simple_bot_reply(bot: &dyn OpenAICompatibleBot, query: &str) -> Result<Reply, String> {
    let req = CallWithToolsRequest {
        messages: vec![json!({"role": "user", "content": query})],
        stream: false,
        model: Some(bot.get_api_config().model),
        ..Default::default()
    };
    let result = bot.call_with_tools(req).await.map_err(|e| e.to_string())?;
    let LlmResult::Complete(body) = result else {
        return Err("expected non-streaming response".into());
    };
    let content = body
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(Reply::text(content))
}
