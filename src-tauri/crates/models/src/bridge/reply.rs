//! `bridge/reply.py`

/// `bridge.reply.ReplyType`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReplyType {
    Text = 1,
    Voice = 2,
    Image = 3,
    ImageUrl = 4,
    VideoUrl = 5,
    File = 6,
    Card = 7,
    InviteRoom = 8,
    Info = 9,
    Error = 10,
    TextForce = 11,
    Video = 12,
    Miniapp = 13,
}

/// `bridge.reply.Reply`
#[derive(Debug, Clone)]
pub struct Reply {
    pub ty: ReplyType,
    pub content: String,
}

impl Reply {
    pub fn new(ty: ReplyType, content: impl Into<String>) -> Self {
        Self {
            ty,
            content: content.into(),
        }
    }

    pub fn info(content: impl Into<String>) -> Self {
        Self::new(ReplyType::Info, content)
    }

    pub fn error(content: impl Into<String>) -> Self {
        Self::new(ReplyType::Error, content)
    }

    pub fn text(content: impl Into<String>) -> Self {
        Self::new(ReplyType::Text, content)
    }
}
