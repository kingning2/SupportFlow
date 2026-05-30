//! `models/openai/` — HTTP client and OpenAI official bot.

pub mod open_ai_bot;
pub mod open_ai_session;
pub mod openai_compat;
pub mod openai_http_client;

pub use open_ai_bot::OpenAiBot;
pub use open_ai_session::OpenAiSession;
pub use openai_compat::OpenAiHttpError;
pub use openai_http_client::{OpenAiHttpClient, DEFAULT_API_BASE, DEFAULT_TIMEOUT_SECS};
