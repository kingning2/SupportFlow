//! `agent/skills/manager.py`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::{debug, warn};

use super::formatter::format_skills_for_prompt;
use super::loader::SkillLoader;
use super::types::{Skill, SkillEntry};

pub struct SkillManager {
    pub builtin_dir: PathBuf,
    pub custom_dir: PathBuf,
    skills: HashMap<String, SkillEntry>,
}

impl SkillManager {
    pub fn new(workspace_dir: impl AsRef<Path>, builtin_dir: Option<PathBuf>) -> Self {
        let workspace = workspace_dir.as_ref().to_path_buf();
        let custom_dir = workspace.join("skills");
        let builtin_dir = builtin_dir.unwrap_or_else(|| workspace.join("skills"));
        let mut mgr = Self {
            builtin_dir,
            custom_dir,
            skills: HashMap::new(),
        };
        mgr.refresh_skills();
        mgr
    }

    pub fn refresh_skills(&mut self) {
        let loader = SkillLoader;
        let result = loader.load_all_skills(&self.builtin_dir, &self.custom_dir);
        self.skills.clear();
        for entry in result.skills {
            self.skills.insert(entry.skill.name.clone(), entry);
        }
        for d in result.diagnostics {
            debug!(%d, "SkillLoader diagnostic");
        }
        debug!(count = self.skills.len(), "SkillManager refreshed");
    }

    pub fn list_skills(&self) -> Vec<&SkillEntry> {
        self.skills.values().collect()
    }

    pub fn get_skill(&self, name: &str) -> Option<&SkillEntry> {
        self.skills.get(name)
    }

    pub fn filter_skills(&self, skill_filter: Option<&[String]>) -> Vec<&SkillEntry> {
        self.skills
            .values()
            .filter(|e| e.enabled)
            .filter(|e| {
                skill_filter
                    .map(|f| f.contains(&e.skill.name))
                    .unwrap_or(true)
            })
            .collect()
    }

    pub fn build_skills_prompt(&self, skill_filter: Option<&[String]>) -> String {
        let eligible: Vec<Skill> = self
            .filter_skills(skill_filter)
            .into_iter()
            .map(|e| e.skill.clone())
            .collect();
        let result = format_skills_for_prompt(&eligible);
        if result.is_empty() {
            warn!("No skills in prompt (eligible count = 0)");
        }
        result
    }
}
