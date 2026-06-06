//! `agent/skills/`

mod config;
mod formatter;
mod frontmatter;
mod installer;
mod loader;
mod manager;
mod types;

pub use config::{
    hub_api_base, load as load_skills_config, register_skill, save as save_skills_config,
    skills_config_path, skills_dir, SkillConfigEntry, SkillsConfigMap,
};
pub use formatter::format_skills_for_prompt;
pub use installer::{install_skill_source, InstallSkillResult};
pub use manager::SkillManager;
pub use types::{Skill, SkillEntry};
