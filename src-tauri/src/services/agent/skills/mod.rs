//! `agent/skills/`

mod config;
mod frontmatter;
mod installer;
mod loader;
mod manager;
mod types;

pub use config::{
    builtin_skills_dir, hub_api_base, is_enabled_in_config, load as load_skills_config,
    merge_disk_skills, register_skill, save as save_skills_config, set_enabled, skills_config_path,
    skills_dir, SkillConfigEntry, SkillsConfigMap,
};
pub use installer::{install_skill_source, InstallSkillResult};
pub use manager::SkillManager;
pub use types::{Skill, SkillEntry};
