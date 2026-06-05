//! `agent/skills/types.py`

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub file_path: String,
    pub base_dir: String,
    pub source: String,
    pub disable_model_invocation: bool,
}

#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub skill: Skill,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct LoadSkillsResult {
    pub skills: Vec<SkillEntry>,
    pub diagnostics: Vec<String>,
}
