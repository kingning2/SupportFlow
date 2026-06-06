//! `agent/tools/web_fetch/`

mod document;
mod html;
#[path = "web_fetch.rs"]
mod web_fetch_tool;

pub use web_fetch_tool::WebFetchTool;
