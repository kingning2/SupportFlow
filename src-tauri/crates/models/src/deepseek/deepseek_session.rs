//! `models/deepseek/deepseek_session.py`

pub use crate::session::{SessionClass, StandardChatSession as DeepSeekSession};

pub fn session_class() -> SessionClass {
    SessionClass::StandardChatBlocks
}
