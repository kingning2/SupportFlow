//! `agent/skills/`

mod formatter;
mod frontmatter;
mod loader;
mod manager;
mod types;

pub use formatter::format_skills_for_prompt;
pub use manager::SkillManager;
pub use types::{Skill, SkillEntry};
