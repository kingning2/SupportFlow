//! `agent/skills/loader.py` (discovery rules).

use std::path::{Path, PathBuf};

use super::frontmatter::{body_after_frontmatter, parse_frontmatter};
use super::types::{LoadSkillsResult, Skill, SkillEntry};

pub struct SkillLoader;

impl SkillLoader {
    pub fn load_all_skills(&self, builtin_dir: &Path, custom_dir: &Path) -> LoadSkillsResult {
        let mut result = LoadSkillsResult::default();
        for (dir, source) in [(builtin_dir, "builtin"), (custom_dir, "custom")] {
            if dir.is_dir() {
                let sub = self.load_skills_from_dir(dir, source);
                result.skills.extend(sub.skills);
                result.diagnostics.extend(sub.diagnostics);
            }
        }
        result
    }

    pub fn load_skills_from_dir(&self, dir_path: &Path, source: &str) -> LoadSkillsResult {
        if !dir_path.is_dir() {
            return LoadSkillsResult {
                diagnostics: vec![format!("Directory does not exist: {}", dir_path.display())],
                ..Default::default()
            };
        }
        self.load_recursive(dir_path, source, true)
    }

    fn load_recursive(
        &self,
        dir_path: &Path,
        source: &str,
        include_root_files: bool,
    ) -> LoadSkillsResult {
        let mut skills = Vec::new();
        let mut diagnostics = Vec::new();

        let entries = match std::fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(e) => {
                diagnostics.push(format!("Failed to list {}: {e}", dir_path.display()));
                return LoadSkillsResult {
                    skills,
                    diagnostics,
                };
            }
        };

        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        if !include_root_files && names.iter().any(|n| n.eq_ignore_ascii_case("SKILL.md")) {
            let skill_md = dir_path.join("SKILL.md");
            if skill_md.is_file() {
                if let Some(entry) = self.load_skill_file(&skill_md, source) {
                    skills.push(entry);
                }
                return LoadSkillsResult {
                    skills,
                    diagnostics,
                };
            }
        }

        for name in names {
            if name.starts_with('.')
                || matches!(
                    name.as_str(),
                    "node_modules" | "__pycache__" | "venv" | ".git"
                )
            {
                continue;
            }
            let full = dir_path.join(&name);
            if full.is_dir() {
                let sub = self.load_recursive(&full, source, false);
                skills.extend(sub.skills);
                diagnostics.extend(sub.diagnostics);
            } else if include_root_files && name.ends_with(".md") && name != "SKILL.md" {
                if let Some(entry) = self.load_skill_file(&full, source) {
                    skills.push(entry);
                }
            }
        }

        LoadSkillsResult {
            skills,
            diagnostics,
        }
    }

    fn load_skill_file(&self, path: &Path, source: &str) -> Option<SkillEntry> {
        let content = std::fs::read_to_string(path).ok()?;
        let fm = parse_frontmatter(&content);
        let body = body_after_frontmatter(&content);

        let name = fm.get("name").cloned().unwrap_or_else(|| {
            path.parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| {
                    path.file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "skill".into())
                })
        });

        let description = fm.get("description").cloned().unwrap_or_else(|| {
            body.lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .chars()
                .take(200)
                .collect()
        });

        let disable = fm
            .get("disable-model-invocation")
            .map(|v| v == "true")
            .unwrap_or(false);

        let base_dir = path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        Some(SkillEntry {
            skill: Skill {
                name,
                description,
                file_path: path.to_string_lossy().into_owned(),
                base_dir,
                source: source.to_string(),
                disable_model_invocation: disable,
            },
            enabled: true,
        })
    }
}

pub fn default_skill_dirs(workspace: &Path) -> (PathBuf, PathBuf) {
    let custom = workspace.join("skills");
    let builtin = workspace.join("skills"); // app may override via config
    (builtin, custom)
}
