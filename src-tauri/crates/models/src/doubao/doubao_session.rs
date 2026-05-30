//! `models/doubao/doubao_session.py`

pub use crate::session::{SessionClass, StandardChatLenSession as DoubaoSession};

pub fn session_class() -> SessionClass {
    SessionClass::StandardChatLen
}
