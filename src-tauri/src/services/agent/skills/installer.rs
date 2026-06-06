use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typeshare::typeshare;

use super::config::{hub_api_base, register_skill, skills_dir};

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSkillResult {
    pub installed_names: Vec<String>,
    pub source: String,
}

/// 安装一个技能来源到工作区。
///
/// # Arguments
///
/// * `workspace` - Agent 工作区根目录
/// * `source` - Skill Hub 名称、GitHub `owner/repo`、zip URL 或本地路径
///
/// # Returns
///
/// * `InstallSkillResult` - 安装后的技能名称与来源信息
pub async fn install_skill_source(workspace: &Path, source: &str) -> Result<InstallSkillResult> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        bail!("skill source is empty");
    }

    if is_local_path(trimmed) {
        return install_local_path(workspace, &expand_tilde(Path::new(trimmed)));
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return install_from_url(workspace, trimmed).await;
    }

    let github_re = Regex::new(r"^[a-zA-Z0-9_\-]+/[a-zA-Z0-9_.\-]+(?:#.+)?$")?;
    if github_re.is_match(trimmed) {
        let (spec, subpath) = trimmed
            .split_once('#')
            .map(|(left, right)| (left, Some(right)))
            .unwrap_or((trimmed, None));
        return install_github_zip(workspace, spec, subpath).await;
    }

    install_from_hub(workspace, trimmed, None).await
}

/// 判断输入是否应按本地路径处理。
///
/// # Arguments
///
/// * `source` - 用户输入的技能来源
///
/// # Returns
///
/// * `bool` - 是否为本地路径
pub fn is_local_path(source: &str) -> bool {
    source.starts_with("./")
        || source.starts_with("../")
        || source.starts_with('/')
        || source.starts_with(".\\")
        || source.starts_with('\\')
        || source
            .as_bytes()
            .get(1)
            .map(|byte| *byte == b':')
            .unwrap_or(false)
}

/// 展开以 `~` 开头的本地路径。
///
/// # Arguments
///
/// * `path` - 原始路径
///
/// # Returns
///
/// * `PathBuf` - 展开后的路径
pub fn expand_tilde(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    if raw == "~" || raw.starts_with("~/") || raw.starts_with("~\\") {
        if let Some(home) = dirs::home_dir() {
            let suffix = raw.trim_start_matches('~').trim_start_matches(['/', '\\']);
            return if suffix.is_empty() {
                home
            } else {
                home.join(suffix)
            };
        }
    }
    path.to_path_buf()
}

async fn install_from_hub(
    workspace: &Path,
    name: &str,
    provider: Option<&str>,
) -> Result<InstallSkillResult> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let mut body = serde_json::Map::new();
    if let Some(value) = provider {
        body.insert("provider".into(), value.into());
    }
    let response = client
        .post(format!("{}/skills/{name}/download", hub_api_base()))
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = response.bytes().await?;

    if content_type.contains("application/json") {
        let data: Value = serde_json::from_slice(&bytes)?;
        if data.get("source_type").and_then(Value::as_str) == Some("github") {
            let url = data["source_url"].as_str().unwrap_or("");
            if let Some(spec) = parse_github_owner_repo(url) {
                let result = install_github_zip(workspace, &spec, None).await?;
                if let Some(display_name) = data["display_name"].as_str() {
                    register_skill(workspace, name, "", "custom", Some(display_name))?;
                }
                return Ok(result);
            }
        }
        bail!("skill hub returned JSON without a supported installer");
    }

    let installed_names = extract_zip_skills(workspace, &bytes, name)?;
    register_skill(workspace, name, "", "custom", None)?;
    Ok(InstallSkillResult {
        installed_names,
        source: name.to_string(),
    })
}

async fn install_github_zip(
    workspace: &Path,
    spec: &str,
    subpath: Option<&str>,
) -> Result<InstallSkillResult> {
    let parts: Vec<_> = spec.split('/').collect();
    if parts.len() != 2 {
        bail!("invalid github spec: {spec}");
    }
    let (owner, repo) = (parts[0], parts[1]);
    let url = format!("https://github.com/{owner}/{repo}/archive/refs/heads/main.zip");
    let bytes = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let label = subpath
        .and_then(|value| value.trim_end_matches('/').rsplit('/').next())
        .unwrap_or(repo);
    let installed_names = extract_zip_skills(workspace, &bytes, label)?;
    register_skill(workspace, label, "", "custom", None)?;
    Ok(InstallSkillResult {
        installed_names,
        source: spec.to_string(),
    })
}

async fn install_from_url(workspace: &Path, url: &str) -> Result<InstallSkillResult> {
    let bytes = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let fallback_name = fallback_name_from_url(url);
    let installed_names = extract_zip_skills(workspace, &bytes, &fallback_name)?;
    Ok(InstallSkillResult {
        installed_names,
        source: url.to_string(),
    })
}

fn install_local_path(workspace: &Path, path: &Path) -> Result<InstallSkillResult> {
    if !path.exists() {
        bail!("path not found: {}", path.display());
    }
    let dest_root = skills_dir(workspace);
    fs_io::create_dir_all(&dest_root)?;

    if path.join("SKILL.md").is_file() {
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "skill".into());
        copy_skill_dir(path, &dest_root.join(&name))?;
        register_skill(workspace, &name, "", "custom", None)?;
        return Ok(InstallSkillResult {
            installed_names: vec![name],
            source: path.display().to_string(),
        });
    }

    let mut installed_names = Vec::new();
    for entry in fs_io::read_dir(path)? {
        let entry = entry?;
        let child_path = entry.path();
        if child_path.is_dir() && child_path.join("SKILL.md").is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            copy_skill_dir(&child_path, &dest_root.join(&name))?;
            register_skill(workspace, &name, "", "custom", None)?;
            installed_names.push(name);
        }
    }

    if installed_names.is_empty() {
        bail!("no SKILL.md found under {}", path.display());
    }

    Ok(InstallSkillResult {
        installed_names,
        source: path.display().to_string(),
    })
}

fn extract_zip_skills(workspace: &Path, bytes: &[u8], fallback_name: &str) -> Result<Vec<String>> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).context("open zip")?;
    let temp = std::env::temp_dir().join(format!("supportflow-skill-{}", std::process::id()));
    if temp.exists() {
        fs::remove_dir_all(&temp).ok();
    }
    fs_io::create_dir_all(&temp)?;
    archive.extract(&temp).context("extract zip")?;

    let mut skill_dirs = Vec::new();
    find_skill_dirs(&temp, &mut skill_dirs);
    if skill_dirs.is_empty() {
        bail!("no SKILL.md found in archive");
    }

    let dest_root = skills_dir(workspace);
    fs_io::create_dir_all(&dest_root)?;

    if skill_dirs.len() == 1 && skill_dirs[0].1 == fallback_name {
        copy_skill_dir(&skill_dirs[0].0, &dest_root.join(fallback_name))?;
        register_skill(workspace, fallback_name, "", "custom", None)?;
        return Ok(vec![fallback_name.to_string()]);
    }

    let mut installed_names = Vec::new();
    for (src, name) in skill_dirs {
        copy_skill_dir(&src, &dest_root.join(&name))?;
        register_skill(workspace, &name, "", "custom", None)?;
        installed_names.push(name);
    }
    Ok(installed_names)
}

fn find_skill_dirs(root: &Path, out: &mut Vec<(PathBuf, String)>) {
    if root.join("SKILL.md").is_file() {
        let name = root
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "skill".into());
        out.push((root.to_path_buf(), name));
        return;
    }

    let Ok(entries) = fs_io::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            find_skill_dirs(&path, out);
        }
    }
}

fn copy_skill_dir(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        fs::remove_dir_all(dest).ok();
    }
    copy_dir_recursive(src, dest)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs_io::create_dir_all(dest)?;
    for entry in fs_io::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs_io::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn parse_github_owner_repo(url: &str) -> Option<String> {
    let re = Regex::new(r"github\.com/([^/]+)/([^/#.]+)").ok()?;
    let cap = re.captures(url)?;
    Some(format!("{}/{}", &cap[1], &cap[2]))
}

fn fallback_name_from_url(url: &str) -> String {
    let parsed = reqwest::Url::parse(url).ok();
    let segment = parsed
        .as_ref()
        .and_then(|value| value.path_segments())
        .and_then(|mut segments| segments.next_back())
        .unwrap_or("skill.zip");
    let stem = segment.strip_suffix(".zip").unwrap_or(segment).trim();
    if stem.is_empty() {
        "skill".into()
    } else {
        stem.to_string()
    }
}
