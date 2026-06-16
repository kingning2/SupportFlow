//! `agent/skills/manager.py`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::{debug, warn};

use super::frontmatter::parse_skill_ref;
use super::loader::SkillLoader;
use super::types::{Skill, SkillEntry};
use crate::services::agent::context::format_skills_for_prompt;

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
            let key = Self::skill_key(&entry.skill.name, &entry.skill.version);
            self.skills.insert(key, entry);
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
        self.resolve_skill(name)
    }

    /// Resolve `name` or `name@version` (custom overrides builtin on same version).
    pub fn resolve_skill(&self, spec: &str) -> Option<&SkillEntry> {
        let (name, version) = parse_skill_ref(spec);
        if let Some(ver) = version {
            return self.skills.get(&Self::skill_key(name, ver));
        }
        self.latest_by_name(name)
    }

    fn skill_key(name: &str, version: &str) -> String {
        format!("{name}@{version}")
    }

    fn latest_by_name(&self, name: &str) -> Option<&SkillEntry> {
        let prefix = format!("{name}@");
        self.skills
            .iter()
            .filter(|(k, _)| k.as_str() == name || k.starts_with(&prefix))
            .max_by(|(ka, a), (kb, b)| {
                version_cmp(&a.skill.version, &b.skill.version).then_with(|| ka.cmp(kb))
            })
            .map(|(_, e)| e)
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

fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| {
        s.split('.')
            .map(|p| p.parse::<u32>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let av = parse(a);
    let bv = parse(b);
    let len = av.len().max(bv.len());
    for i in 0..len {
        let ai = av.get(i).copied().unwrap_or(0);
        let bi = bv.get(i).copied().unwrap_or(0);
        match ai.cmp(&bi) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}
