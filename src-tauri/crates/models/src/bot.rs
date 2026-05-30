//! `models/bot.py` — base trait for chat bots.

use async_trait::async_trait;
use thiserror::Error;

pub use crate::bridge::{Context, Reply, ReplyType};

/// Base bot trait (mirrors Python `Bot.reply`).
#[async_trait]
pub trait Bot: Send + Sync {
    async fn reply(&self, query: &str, context: Option<&Context>) -> Result<Reply, BotError>;
}

#[derive(Debug, Error)]
pub enum BotError {
    #[error("not implemented: {0}")]
    NotImplemented(String),
    #[error("{0}")]
    Message(String),
}
