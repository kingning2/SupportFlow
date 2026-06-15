//! `bridge/bridge.py` — bot routing, voice/translate stubs, agent delegation.

use std::sync::Arc;

use crate::config::{Context, ModelsConfig, Reply};
use tracing::info;

use crate::services::agent::rig::run_simple_chat;

use super::agent_bridge::AgentBridge;
use super::bot_router::auto_pick_voice_to_text;

/// SupportFlow Agent `Bridge` (without Python voice/translate backends).
pub struct Bridge {
    pub config: Arc<ModelsConfig>,
    voice_to_text_provider: String,
    text_to_voice_provider: String,
    agent_bridge: std::sync::Mutex<Option<Arc<AgentBridge>>>,
}

impl Bridge {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
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
            voice_to_text_provider: voice_to_text,
            text_to_voice_provider: text_to_voice,
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
        info!(
            "[Bridge] voice refreshed: voice_to_text={}, text_to_voice={}",
            voice_to_text, text_to_voice
        );
    }

    pub fn reset_bot(&mut self) {
        *self = Self::new(self.config.clone());
    }

    /// Non-agent chat completion — always routed through rig.
    pub async fn fetch_reply_content(&self, query: &str, _context: Option<&Context>) -> Reply {
        match run_simple_chat(self.config.as_ref(), query).await {
            Ok(text) => Reply::text(text),
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
