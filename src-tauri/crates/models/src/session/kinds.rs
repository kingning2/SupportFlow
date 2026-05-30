//! Provider-specific session types (`*_session.py`).

use serde_json::{json, Value};

use super::base::{openai_session_prompt, BaseSession};
use super::discard::{
    discard_exceeding_baidu_wenxin, discard_exceeding_minimax, discard_exceeding_openai_legacy,
    discard_exceeding_standard,
};
use super::tokens::{
    num_tokens_content_blocks, num_tokens_dashscope, num_tokens_from_messages_chatgpt,
    num_tokens_from_string, num_tokens_len_content, num_tokens_minimax,
};

/// Which session implementation to use (maps to Python `SessionManager(sessioncls)`).
#[derive(Debug, Clone, Copy)]
pub enum SessionClass {
    /// `deepseek_session` / `qianfan_session` token counting
    StandardChatBlocks,
    /// `doubao_session` / `moonshot_session` / `zhipu_ai_session` …
    StandardChatLen,
    ChatGpt,
    OpenAi,
    BaiduWenxin,
    Minimax,
    Dashscope,
}

/// Mutable chat session (mirrors Python session instances in the manager dict).
#[derive(Debug, Clone)]
pub enum ChatSession {
    StandardChatBlocks(StandardChatSession),
    StandardChatLen(StandardChatLenSession),
    ChatGpt(ChatGptSession),
    OpenAi(OpenAiSession),
    BaiduWenxin(BaiduWenxinSession),
    Minimax(MinimaxSession),
    Dashscope(DashscopeSession),
}

impl ChatSession {
    pub fn new(
        class: SessionClass,
        session_id: impl Into<String>,
        system_prompt: Option<String>,
        model: &str,
        default_desc: &str,
    ) -> Self {
        let sid = session_id.into();
        match class {
            SessionClass::StandardChatBlocks => Self::StandardChatBlocks(StandardChatSession::new(
                sid,
                system_prompt,
                model,
                default_desc,
            )),
            SessionClass::StandardChatLen => Self::StandardChatLen(StandardChatLenSession::new(
                sid,
                system_prompt,
                model,
                default_desc,
            )),
            SessionClass::ChatGpt => {
                Self::ChatGpt(ChatGptSession::new(sid, system_prompt, model, default_desc))
            }
            SessionClass::OpenAi => {
                Self::OpenAi(OpenAiSession::new(sid, system_prompt, model, default_desc))
            }
            SessionClass::BaiduWenxin => Self::BaiduWenxin(BaiduWenxinSession::new(
                sid,
                system_prompt,
                model,
                default_desc,
            )),
            SessionClass::Minimax => {
                Self::Minimax(MinimaxSession::new(sid, system_prompt, model, default_desc))
            }
            SessionClass::Dashscope => Self::Dashscope(DashscopeSession::new(
                sid,
                system_prompt,
                model,
                default_desc,
            )),
        }
    }

    pub fn set_system_prompt(&mut self, system_prompt: impl Into<String>) {
        match self {
            Self::StandardChatBlocks(s) => s.base.set_system_prompt(system_prompt),
            Self::StandardChatLen(s) => s.base.set_system_prompt(system_prompt),
            Self::ChatGpt(s) => s.base.set_system_prompt(system_prompt),
            Self::OpenAi(s) => s.base.set_system_prompt(system_prompt),
            Self::BaiduWenxin(s) => s.base.system_prompt = system_prompt.into(),
            Self::Minimax(s) => s.base.system_prompt = system_prompt.into(),
            Self::Dashscope(s) => s.base.set_system_prompt(system_prompt),
        }
    }

    pub fn add_query(&mut self, query: &str) {
        match self {
            Self::Minimax(s) => s.add_query(query),
            _ => self.base_mut().add_query(query),
        }
    }

    pub fn add_reply(&mut self, reply: &str) {
        match self {
            Self::Minimax(s) => s.add_reply(reply),
            _ => self.base_mut().add_reply(reply),
        }
    }

    pub fn messages(&self) -> &[Value] {
        match self {
            Self::StandardChatBlocks(s) => &s.base.messages,
            Self::StandardChatLen(s) => &s.base.messages,
            Self::ChatGpt(s) => &s.base.messages,
            Self::OpenAi(s) => &s.base.messages,
            Self::BaiduWenxin(s) => &s.base.messages,
            Self::Minimax(s) => &s.base.messages,
            Self::Dashscope(s) => &s.base.messages,
        }
    }

    pub fn discard_exceeding(&mut self, max_tokens: u32, cur_tokens: Option<u32>) -> u32 {
        match self {
            Self::StandardChatBlocks(s) => s.discard_exceeding(max_tokens, cur_tokens),
            Self::StandardChatLen(s) => s.discard_exceeding(max_tokens, cur_tokens),
            Self::ChatGpt(s) => s.discard_exceeding(max_tokens, cur_tokens),
            Self::OpenAi(s) => s.discard_exceeding(max_tokens, cur_tokens),
            Self::BaiduWenxin(s) => s.discard_exceeding(max_tokens, cur_tokens),
            Self::Minimax(s) => s.discard_exceeding(max_tokens, cur_tokens),
            Self::Dashscope(s) => s.discard_exceeding(max_tokens, cur_tokens),
        }
    }

    fn base_mut(&mut self) -> &mut BaseSession {
        match self {
            Self::StandardChatBlocks(s) => &mut s.base,
            Self::StandardChatLen(s) => &mut s.base,
            Self::ChatGpt(s) => &mut s.base,
            Self::OpenAi(s) => &mut s.base,
            Self::BaiduWenxin(s) => &mut s.base,
            Self::Dashscope(s) => &mut s.base,
            Self::Minimax(_) => panic!("minimax uses custom messages"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StandardChatLenSession {
    pub base: BaseSession,
    pub model: String,
}

impl StandardChatLenSession {
    pub fn new(
        session_id: String,
        system_prompt: Option<String>,
        model: &str,
        default_desc: &str,
    ) -> Self {
        Self {
            base: BaseSession::new(session_id, system_prompt, default_desc),
            model: model.to_string(),
        }
    }

    pub fn discard_exceeding(&mut self, max_tokens: u32, cur_tokens: Option<u32>) -> u32 {
        let model = self.model.clone();
        discard_exceeding_standard(&mut self.base.messages, max_tokens, cur_tokens, |msgs| {
            num_tokens_len_content(msgs, &model)
        })
    }
}

#[derive(Debug, Clone)]
pub struct StandardChatSession {
    pub base: BaseSession,
    pub model: String,
}

impl StandardChatSession {
    pub fn new(
        session_id: String,
        system_prompt: Option<String>,
        model: &str,
        default_desc: &str,
    ) -> Self {
        Self {
            base: BaseSession::new(session_id, system_prompt, default_desc),
            model: model.to_string(),
        }
    }

    pub fn discard_exceeding(&mut self, max_tokens: u32, cur_tokens: Option<u32>) -> u32 {
        let model = self.model.clone();
        discard_exceeding_standard(&mut self.base.messages, max_tokens, cur_tokens, |msgs| {
            num_tokens_content_blocks(msgs, &model)
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChatGptSession {
    pub base: BaseSession,
    pub model: String,
}

impl ChatGptSession {
    pub fn new(
        session_id: String,
        system_prompt: Option<String>,
        model: &str,
        default_desc: &str,
    ) -> Self {
        Self {
            base: BaseSession::new(session_id, system_prompt, default_desc),
            model: model.to_string(),
        }
    }

    pub fn discard_exceeding(&mut self, max_tokens: u32, cur_tokens: Option<u32>) -> u32 {
        let model = self.model.clone();
        discard_exceeding_standard(&mut self.base.messages, max_tokens, cur_tokens, |msgs| {
            num_tokens_from_messages_chatgpt(msgs, &model)
        })
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiSession {
    pub base: BaseSession,
    pub model: String,
}

impl OpenAiSession {
    pub fn new(
        session_id: String,
        system_prompt: Option<String>,
        model: &str,
        default_desc: &str,
    ) -> Self {
        Self {
            base: BaseSession::new(session_id, system_prompt, default_desc),
            model: model.to_string(),
        }
    }

    pub fn discard_exceeding(&mut self, max_tokens: u32, cur_tokens: Option<u32>) -> u32 {
        let model = self.model.clone();
        discard_exceeding_openai_legacy(
            &mut self.base.messages,
            max_tokens,
            cur_tokens,
            |msgs| num_tokens_from_string(&openai_session_prompt(msgs), &model),
            |msgs| openai_session_prompt(msgs).chars().count() as u32,
        )
    }
}

#[derive(Debug, Clone)]
pub struct BaiduWenxinSession {
    pub base: BaseSession,
    pub model: String,
}

impl BaiduWenxinSession {
    pub fn new(
        session_id: String,
        system_prompt: Option<String>,
        model: &str,
        default_desc: &str,
    ) -> Self {
        // 百度文心不支持 system prompt — 不 reset（与 Python 一致）
        let system_prompt = system_prompt.unwrap_or_else(|| default_desc.to_string());
        Self {
            base: BaseSession {
                session_id: session_id.into(),
                system_prompt,
                messages: Vec::new(),
            },
            model: model.to_string(),
        }
    }

    pub fn discard_exceeding(&mut self, max_tokens: u32, cur_tokens: Option<u32>) -> u32 {
        let model = self.model.clone();
        discard_exceeding_baidu_wenxin(&mut self.base.messages, max_tokens, cur_tokens, |msgs| {
            super::tokens::num_tokens_baidu_wenxin(msgs, &model)
        })
    }
}

#[derive(Debug, Clone)]
pub struct MinimaxSession {
    pub base: BaseSession,
    pub model: String,
}

impl MinimaxSession {
    pub fn new(
        session_id: String,
        system_prompt: Option<String>,
        model: &str,
        default_desc: &str,
    ) -> Self {
        let system_prompt = system_prompt.unwrap_or_else(|| default_desc.to_string());
        Self {
            base: BaseSession {
                session_id: session_id.into(),
                system_prompt,
                messages: Vec::new(),
            },
            model: model.to_string(),
        }
    }

    pub fn add_query(&mut self, query: &str) {
        self.base.messages.push(json!({
            "sender_type": "USER",
            "sender_name": self.base.session_id,
            "text": query,
        }));
    }

    pub fn add_reply(&mut self, reply: &str) {
        self.base.messages.push(json!({
            "sender_type": "BOT",
            "sender_name": "MM智能助理",
            "text": reply,
        }));
    }

    pub fn discard_exceeding(&mut self, max_tokens: u32, cur_tokens: Option<u32>) -> u32 {
        let model = self.model.clone();
        discard_exceeding_minimax(&mut self.base.messages, max_tokens, cur_tokens, |msgs| {
            num_tokens_minimax(msgs, &model)
        })
    }
}

#[derive(Debug, Clone)]
pub struct DashscopeSession {
    pub base: BaseSession,
}

impl DashscopeSession {
    pub fn new(
        session_id: String,
        _system_prompt: Option<String>,
        _model: &str,
        default_desc: &str,
    ) -> Self {
        // Python: super().__init__(session_id) only — then reset()
        let mut base = BaseSession::new(session_id, None, default_desc);
        base.reset();
        Self { base }
    }

    pub fn discard_exceeding(&mut self, max_tokens: u32, cur_tokens: Option<u32>) -> u32 {
        discard_exceeding_standard(&mut self.base.messages, max_tokens, cur_tokens, |msgs| {
            num_tokens_dashscope(msgs)
        })
    }
}
