//! `models/chatgpt/chat_gpt_session.py`

pub use crate::session::{ChatGptSession, SessionClass};

pub fn session_class() -> SessionClass {
    SessionClass::ChatGpt
}
