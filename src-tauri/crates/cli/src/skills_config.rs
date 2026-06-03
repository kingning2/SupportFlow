//! `skills/skills_config.json` — parity with SupportFlow Agent Python CLI.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfigEntry {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

fn default_source() -> String {
    "custom".into()
}
fn default_true() -> bool {
    true
}
fn default_category() -> String {
    "skill".into()
}

pub type SkillsConfigMap = HashMap<String, SkillConfigEntry>;

pub fn load(workspace: &Path) -> SkillsConfigMap {
    let path = paths::skills_config_path(workspace);
    if !path.is_file() {
        return HashMap::new();
    }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save(workspace: &Path, config: &SkillsConfigMap) -> Result<()> {
    let dir = paths::skills_dir(workspace);
    fs::create_dir_all(&dir)?;
    let path = paths::skills_config_path(workspace);
    let json = serde_json::to_string_pretty(config).context("serialize skills_config")?;
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))
}

pub fn set_enabled(workspace: &Path, name: &str, enabled: bool) -> Result<()> {
    let mut cfg = load(workspace);
    let entry = cfg
        .get_mut(name)
        .ok_or_else(|| anyhow::anyhow!("skill '{name}' not in skills_config.json"))?;
    entry.enabled = enabled;
    save(workspace, &cfg)
}

pub fn register_skill(
    workspace: &Path,
    name: &str,
    description: &str,
    source: &str,
    display_name: Option<&str>,
) -> Result<()> {
    let mut cfg = load(workspace);
    cfg.insert(
        name.to_string(),
        SkillConfigEntry {
            name: name.to_string(),
            description: description.to_string(),
            source: source.to_string(),
            enabled: true,
            category: "skill".into(),
            display_name: display_name.map(str::to_string),
        },
    );
    save(workspace, &cfg)
}

pub fn merge_disk_skills(workspace: &Path, config: &mut SkillsConfigMap) -> bool {
    let mut dirty = false;
    for (dir, source) in [
        (paths::skills_dir(workspace), "custom"),
        (builtin_skills_dir(), "builtin"),
    ] {
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "skills_config.json" {
                continue;
            }
            if !path.join("SKILL.md").is_file() {
                continue;
            }
            if config.contains_key(&name) {
                continue;
            }
            let desc = read_skill_description(&path);
            config.insert(
                name.clone(),
                SkillConfigEntry {
                    name: name.clone(),
                    description: desc,
                    source: source.into(),
                    enabled: true,
                    category: "skill".into(),
                    display_name: None,
                },
            );
            dirty = true;
        }
    }
    dirty
}

fn builtin_skills_dir() -> PathBuf {
    // Repo `skills/` when developing from tauri-template
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../skills");
    if repo.is_dir() {
        return repo;
    }
    PathBuf::new()
}

fn read_skill_description(skill_path: &Path) -> String {
    let skill_md = skill_path.join("SKILL.md");
    let Ok(content) = fs::read_to_string(&skill_md) else {
        return String::new();
    };
    for line in content.lines() {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') && !t.starts_with("---") {
            return t.chars().take(200).collect();
        }
    }
    String::new()
}

pub fn is_enabled_in_config(config: &SkillsConfigMap, name: &str) -> bool {
    config.get(name).map(|e| e.enabled).unwrap_or(true)
}

pub fn hub_api_base() -> &'static str {
    "https://skills.supportflow.ai/api"
}
