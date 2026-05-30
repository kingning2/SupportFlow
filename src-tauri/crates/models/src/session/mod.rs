//! Session layer — mirrors `models/session_manager.py` and `models/**/**_session.py`.

mod base;
mod discard;
mod expired_dict;
mod kinds;
mod manager;
mod tokens;

pub use base::BaseSession;
pub use kinds::{
    BaiduWenxinSession, ChatGptSession, ChatSession, DashscopeSession, MinimaxSession,
    OpenAiSession, SessionClass, StandardChatLenSession, StandardChatSession,
};
pub use manager::SessionManager;
