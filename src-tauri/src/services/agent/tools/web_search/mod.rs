//! `agent/tools/web_search/`

mod config;
#[path = "web_search.rs"]
mod web_search_tool;

pub use config::WebSearchSettings;
pub use web_search_tool::WebSearchTool;
