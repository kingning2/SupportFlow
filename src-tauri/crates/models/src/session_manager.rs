//! Re-export session types (Python `session_manager.py` + `*_session.py`).

pub use crate::session::{
    BaiduWenxinSession, BaseSession, ChatGptSession, ChatSession, DashscopeSession, MinimaxSession,
    OpenAiSession, SessionClass, SessionManager, StandardChatLenSession, StandardChatSession,
};

/// Alias for backward compatibility in this crate.
pub type Session = ChatSession;
