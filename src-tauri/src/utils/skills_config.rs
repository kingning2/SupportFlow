use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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

/// 获取工作区 skills 目录路径。
///
/// # Arguments
///
/// * `workspace` - Agent 工作区根目录
///
/// # Returns
///
/// * `PathBuf` - skills 目录绝对路径
pub fn skills_dir(workspace: &Path) -> PathBuf {
    workspace.join("skills")
}

/// 获取 skills 配置文件路径。
///
/// # Arguments
///
/// * `workspace` - Agent 工作区根目录
///
/// # Returns
///
/// * `PathBuf` - `skills_config.json` 文件路径
pub fn skills_config_path(workspace: &Path) -> PathBuf {
    skills_dir(workspace).join("skills_config.json")
}

/// 读取工作区中的 skills 配置。
///
/// # Arguments
///
/// * `workspace` - Agent 工作区根目录
///
/// # Returns
///
/// * `SkillsConfigMap` - 已保存的技能配置映射
pub fn load(workspace: &Path) -> SkillsConfigMap {
    let path = skills_config_path(workspace);
    if !path.is_file() {
        return HashMap::new();
    }
    let raw = fs_io::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

/// 保存工作区中的 skills 配置。
///
/// # Arguments
///
/// * `workspace` - Agent 工作区根目录
/// * `config` - 待写入的技能配置映射
///
/// # Returns
///
/// * `Result<(), anyhow::Error>` - 保存结果
pub fn save(workspace: &Path, config: &SkillsConfigMap) -> Result<()> {
    let dir = skills_dir(workspace);
    fs_io::create_dir_all(&dir)?;
    let path = skills_config_path(workspace);
    let json = serde_json::to_string_pretty(config).context("serialize skills_config")?;
    fs_io::write(&path, json).with_context(|| format!("write {}", path.display()))
}

/// 注册或更新一个技能条目。
///
/// # Arguments
///
/// * `workspace` - Agent 工作区根目录
/// * `name` - 技能名称
/// * `description` - 技能描述
/// * `source` - 技能来源标识
/// * `display_name` - 可选展示名称
///
/// # Returns
///
/// * `Result<(), anyhow::Error>` - 写入结果
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

/// 返回 Skill Hub API 根地址。
///
/// # Returns
///
/// * `&'static str` - Skill Hub API 地址
pub fn hub_api_base() -> &'static str {
    "https://skills.supportflow.ai/api"
}
