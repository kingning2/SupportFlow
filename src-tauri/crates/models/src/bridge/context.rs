//! `bridge/context.py`

use std::collections::HashMap;

/// `bridge.context.ContextType`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ContextType {
    Text = 1,
    Voice = 2,
    Image = 3,
    File = 4,
    Video = 5,
    Sharing = 6,
    ImageCreate = 10,
    AcceptFriend = 19,
    JoinGroup = 20,
    Patpat = 21,
    Function = 22,
    ExitGroup = 23,
}

/// `bridge.context.Context`
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub ty: Option<ContextType>,
    pub content: Option<String>,
    pub kwargs: HashMap<String, String>,
}

impl Context {
    pub fn new(ty: ContextType, content: impl Into<String>) -> Self {
        Self {
            ty: Some(ty),
            content: Some(content.into()),
            kwargs: HashMap::new(),
        }
    }

    pub fn text(query: impl Into<String>, session_id: impl Into<String>) -> Self {
        let mut ctx = Self::new(ContextType::Text, "");
        ctx.content = Some(query.into());
        ctx.kwargs.insert("session_id".into(), session_id.into());
        ctx
    }

    pub fn get_type(&self) -> Option<ContextType> {
        self.ty
    }

    pub fn session_id(&self) -> Option<&str> {
        self.kwargs.get("session_id").map(String::as_str)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.kwargs.get(key).map(String::as_str)
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.kwargs.insert(key.into(), value.into());
    }
}
