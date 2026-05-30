//! `models/openai/open_ai_session.py`

pub use crate::session::{OpenAiSession, SessionClass};

pub fn session_class() -> SessionClass {
    SessionClass::OpenAi
}
