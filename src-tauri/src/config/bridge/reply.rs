//! `bridge/reply.py`

/// `bridge.reply.ReplyType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
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

impl ReplyType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::Voice => "VOICE",
            Self::Image => "IMAGE",
            Self::ImageUrl => "IMAGE_URL",
            Self::VideoUrl => "VIDEO_URL",
            Self::File => "FILE",
            Self::Card => "CARD",
            Self::InviteRoom => "INVITE_ROOM",
            Self::Info => "INFO",
            Self::Error => "ERROR",
            Self::TextForce => "TEXT_FORCE",
            Self::Video => "VIDEO",
            Self::Miniapp => "MINIAPP",
        }
    }
}

/// `bridge.reply.Reply`
#[derive(Debug, Clone, serde::Serialize)]
pub struct Reply {
    pub ty: ReplyType,
    pub content: String,
    /// Accompanying text for IMAGE_URL / FILE replies (Python dynamic attr).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

impl Reply {
    pub fn new(ty: ReplyType, content: impl Into<String>) -> Self {
        Self {
            ty,
            content: content.into(),
            text_content: None,
            file_name: None,
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "content": self.content,
            "reply_type": self.ty.as_str(),
            "text_content": self.text_content,
            "file_name": self.file_name,
        })
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
