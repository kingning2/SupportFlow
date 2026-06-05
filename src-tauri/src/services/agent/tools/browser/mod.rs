//! Browser automation for AI agents — pure Rust via [chromiumoxide](https://crates.io/crates/chromiumoxide) (CDP).

mod browser_tool;
mod config;
mod service;
mod snapshot;

pub use browser_tool::BrowserTool;
pub use config::{resolve_chrome_executable, BrowserSettings};
