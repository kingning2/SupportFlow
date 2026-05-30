//! `models/session_manager.py` — `SessionManager`.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tracing::debug;

use super::expired_dict::ExpiredDict;
use super::kinds::{ChatSession, SessionClass};
use crate::config::ModelsConfig;

/// In-memory session registry (mirrors Python `SessionManager`).
#[derive(Debug)]
pub struct SessionManager {
    sessions: Arc<RwLock<SessionStore>>,
    session_class: SessionClass,
    default_model: String,
    default_desc: String,
    conversation_max_tokens: u32,
}

#[derive(Debug)]
enum SessionStore {
    Plain(HashMap<String, ChatSession>),
    Expiring(ExpiredDict<ChatSession>),
}

impl SessionManager {
    pub fn new(config: &ModelsConfig, session_class: SessionClass, default_model: &str) -> Self {
        let sessions = if let Some(secs) = config.expires_in_seconds {
            SessionStore::Expiring(ExpiredDict::new(secs))
        } else {
            SessionStore::Plain(HashMap::new())
        };
        Self {
            sessions: Arc::new(RwLock::new(sessions)),
            session_class,
            default_model: config.model_or(default_model),
            default_desc: config.character_desc.clone().unwrap_or_default(),
            conversation_max_tokens: config.conversation_max_tokens.unwrap_or(1000),
        }
    }

    pub fn build_session(
        &self,
        session_id: Option<&str>,
        system_prompt: Option<&str>,
    ) -> ChatSession {
        if session_id.is_none() {
            return ChatSession::new(
                self.session_class,
                "",
                system_prompt.map(str::to_string),
                &self.default_model,
                &self.default_desc,
            );
        }
        let sid = session_id.unwrap();
        let mut guard = self.sessions.write().expect("session lock");

        if !contains_session(&mut guard, sid) {
            insert_session(
                &mut guard,
                sid.to_string(),
                ChatSession::new(
                    self.session_class,
                    sid,
                    system_prompt.map(str::to_string),
                    &self.default_model,
                    &self.default_desc,
                ),
            );
        } else if let Some(prompt) = system_prompt {
            if let Some(session) = get_session_mut(&mut guard, sid) {
                session.set_system_prompt(prompt);
            }
        }

        get_session_mut(&mut guard, sid)
            .expect("session must exist after build")
            .clone()
    }

    pub fn session_query(&self, query: &str, session_id: &str) -> ChatSession {
        let mut guard = self.sessions.write().expect("session lock");
        ensure_session(
            &mut guard,
            session_id,
            self.session_class,
            &self.default_model,
            &self.default_desc,
        );
        let session = get_session_mut(&mut guard, session_id).expect("session");
        session.add_query(query);
        let max_tokens = self.conversation_max_tokens;
        let total_tokens = session.discard_exceeding(max_tokens, None);
        debug!(prompt_tokens_used = total_tokens, "session_query");
        session.clone()
    }

    pub fn session_reply(
        &self,
        reply: &str,
        session_id: &str,
        total_tokens: Option<u32>,
    ) -> ChatSession {
        let mut guard = self.sessions.write().expect("session lock");
        ensure_session(
            &mut guard,
            session_id,
            self.session_class,
            &self.default_model,
            &self.default_desc,
        );
        let session = get_session_mut(&mut guard, session_id).expect("session");
        session.add_reply(reply);
        let max_tokens = self.conversation_max_tokens;
        let tokens_cnt = session.discard_exceeding(max_tokens, total_tokens);
        debug!(
            ?total_tokens,
            savesession_tokens = tokens_cnt,
            "session_reply"
        );
        session.clone()
    }

    pub fn clear_session(&self, session_id: &str) {
        let mut guard = self.sessions.write().expect("session lock");
        remove_session(&mut guard, session_id);
    }

    pub fn clear_all_sessions(&self) {
        let mut guard = self.sessions.write().expect("session lock");
        match &mut *guard {
            SessionStore::Plain(m) => m.clear(),
            SessionStore::Expiring(e) => e.clear(),
        }
    }
}

fn ensure_session(
    store: &mut SessionStore,
    session_id: &str,
    class: SessionClass,
    default_model: &str,
    default_desc: &str,
) {
    if !contains_session(store, session_id) {
        insert_session(
            store,
            session_id.to_string(),
            ChatSession::new(class, session_id, None, default_model, default_desc),
        );
    }
}

fn contains_session(store: &mut SessionStore, key: &str) -> bool {
    match store {
        SessionStore::Plain(m) => m.contains_key(key),
        SessionStore::Expiring(e) => e.contains_key(key),
    }
}

fn get_session_mut<'a>(store: &'a mut SessionStore, key: &str) -> Option<&'a mut ChatSession> {
    match store {
        SessionStore::Plain(m) => m.get_mut(key),
        SessionStore::Expiring(e) => e.get_mut(key),
    }
}

fn insert_session(store: &mut SessionStore, key: String, value: ChatSession) {
    match store {
        SessionStore::Plain(m) => {
            m.insert(key, value);
        }
        SessionStore::Expiring(e) => e.insert(key, value),
    }
}

fn remove_session(store: &mut SessionStore, key: &str) {
    match store {
        SessionStore::Plain(m) => {
            m.remove(key);
        }
        SessionStore::Expiring(e) => {
            e.remove(key);
        }
    }
}
