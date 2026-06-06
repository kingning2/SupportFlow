//! `agent/tools/vision/`

mod config;
#[path = "vision.rs"]
mod vision_tool;

pub use vision_tool::VisionTool;
