use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::services::agent::SkillManager;
use anyhow::{bail, Context, Result};
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;

use crate::cli::paths;
use crate::services::agent::skills::{
    hub_api_base, is_enabled_in_config, load_skills_config, merge_disk_skills, register_skill,
    save_skills_config, set_enabled, skills_dir, SkillConfigEntry,
};

const REMOTE_PAGE_SIZE: u32 = 10;

#[derive(clap::Subcommand)]
pub enum SkillCommand {
    /// List installed skills or browse Skill Hub
    List {
        #[arg(long)]
        remote: bool,
        #[arg(long, default_value_t = 1)]
        page: u32,
    },
    /// Search Skill Hub
    Search { query: String },
    /// Install from Skill Hub, GitHub owner/repo, or local path
    Install { name: String },
    /// Enable a skill in skills_config.json
    Enable { name: String },
    /// Disable a skill
    Disable { name: String },
    /// Show skill details
    Info { name: String },
}

pub fn run_command(command: SkillCommand) -> Result<()> {
    match command {
        SkillCommand::List { remote, page } => {
            if remote {
                list_remote(page)?;
            } else {
                list_local()?;
            }
        }
        SkillCommand::Search { query } => search(&query)?,
        SkillCommand::Install { name } => install(&name)?,
        SkillCommand::Enable { name } => {
            let ws = paths::resolve_workspace()?;
            set_enabled(&ws, &name, true)?;
            println!("Enabled skill '{name}'.");
        }
        SkillCommand::Disable { name } => {
            let ws = paths::resolve_workspace()?;
            set_enabled(&ws, &name, false)?;
            println!("Disabled skill '{name}'.");
        }
        SkillCommand::Info { name } => info(&name)?,
    }
    Ok(())
}

fn list_local() -> Result<()> {
    let ws = paths::resolve_workspace()?;
    let mut config = load_skills_config(&ws);
    if merge_disk_skills(&ws, &mut config) {
        save_skills_config(&ws, &config)?;
    }
    if config.is_empty() {
        println!("No skills installed.");
        return Ok(());
    }
    let mut entries: Vec<_> = config.values().collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    print_skill_table(entries, "Installed skills");
    Ok(())
}

fn list_remote(page: u32) -> Result<()> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let resp: Value = client
        .get(format!("{}/skills", hub_api_base()))
        .query(&[("page", page), ("limit", REMOTE_PAGE_SIZE)])
        .send()?
        .error_for_status()?
        .json()?;

    let skills = resp["skills"].as_array().cloned().unwrap_or_default();
    let total = resp["total"].as_u64().unwrap_or(skills.len() as u64) as u32;
    if skills.is_empty() && page == 1 {
        println!("No skills available on Skill Hub.");
        return Ok(());
    }
    let total_pages = std::cmp::max(1, total.div_ceil(REMOTE_PAGE_SIZE));
    let page = page.min(total_pages);
    let installed: std::collections::HashSet<_> = load_skills_config(&paths::resolve_workspace()?)
        .keys()
        .cloned()
        .collect();

    println!("\n  Skill Hub ({total} available) — page {page}/{total_pages}\n");
    for s in &skills {
        let name = s["name"].as_str().unwrap_or("");
        let desc = s["description"]
            .as_str()
            .or_else(|| s["display_name"].as_str())
            .unwrap_or("");
        let desc = truncate(desc, 50);
        let status = if installed.contains(name) {
            "installed"
        } else {
            "—"
        };
        println!("  {name:<24} {status:<12} {desc}");
    }
    println!("\n  Install:  sf skill install <name>");
    println!("  Browse:   https://skills.supportflow.ai\n");
    Ok(())
}

fn search(query: &str) -> Result<()> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let resp: Value = client
        .get(format!("{}/skills/search", hub_api_base()))
        .query(&[("q", query)])
        .send()?
        .error_for_status()?
        .json()?;
    let skills = resp["skills"].as_array().cloned().unwrap_or_default();
    if skills.is_empty() {
        println!("No skills found for \"{query}\".");
        return Ok(());
    }
    println!(
        "\n  Search results for \"{query}\" ({} found)\n",
        skills.len()
    );
    for s in &skills {
        let name = s["name"].as_str().unwrap_or("");
        let desc = truncate(
            s["description"]
                .as_str()
                .or_else(|| s["display_name"].as_str())
                .unwrap_or(""),
            50,
        );
        println!("  {name:<24} {desc}");
    }
    println!("\n  Install with: sf skill install <name>\n");
    Ok(())
}

fn install(name: &str) -> Result<()> {
    if name.starts_with("./") || name.starts_with("../") || name.starts_with('/') {
        install_local_path(Path::new(name))?;
        return Ok(());
    }
    #[cfg(windows)]
    if name.starts_with(r".\") || name.len() > 2 && name.as_bytes()[1] == b':' {
        install_local_path(Path::new(name))?;
        return Ok(());
    }

    if name.starts_with("http://") || name.starts_with("https://") {
        bail!("URL install: use owner/repo or Skill Hub name (full URL install coming soon)");
    }

    let github_re = Regex::new(r"^[a-zA-Z0-9_\-]+/[a-zA-Z0-9_.\-]+(?:#.+)?$").unwrap();
    if github_re.is_match(name) {
        let (spec, subpath) = name
            .split_once('#')
            .map(|(a, b)| (a, Some(b)))
            .unwrap_or((name, None));
        install_github_zip(spec, subpath)?;
        return Ok(());
    }

    install_from_hub(name, None)?;
    Ok(())
}

fn install_from_hub(name: &str, provider: Option<&str>) -> Result<()> {
    println!("Fetching skill '{name}' from Skill Hub...");
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let mut body = serde_json::Map::new();
    if let Some(p) = provider {
        body.insert("provider".into(), p.into());
    }
    let resp = client
        .post(format!("{}/skills/{name}/download", hub_api_base()))
        .json(&body)
        .send()?
        .error_for_status()?;

    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let bytes = resp.bytes().context("download body")?;

    if ct.contains("application/json") {
        let data: Value = serde_json::from_slice(&bytes)?;
        if data.get("source_type").and_then(|v| v.as_str()) == Some("github") {
            let url = data["source_url"].as_str().unwrap_or("");
            if let Some(spec) = parse_github_owner_repo(url) {
                install_github_zip(&spec, None)?;
                if let Some(dn) = data["display_name"].as_str() {
                    let ws = paths::resolve_workspace()?;
                    register_skill(&ws, name, "", "custom", Some(dn))?;
                }
                return Ok(());
            }
        }
        bail!("Skill Hub returned JSON without a supported installer");
    }

    extract_zip_skills(&bytes, name)?;
    let ws = paths::resolve_workspace()?;
    register_skill(&ws, name, "", "custom", None)?;
    println!("✓ Installed skill '{name}'");
    Ok(())
}

fn install_github_zip(spec: &str, subpath: Option<&str>) -> Result<()> {
    let parts: Vec<_> = spec.split('/').collect();
    if parts.len() != 2 {
        bail!("invalid github spec: {spec}");
    }
    let (owner, repo) = (parts[0], parts[1]);
    let url = format!("https://github.com/{owner}/{repo}/archive/refs/heads/main.zip");
    println!("Downloading {owner}/{repo}...");
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let bytes = client.get(&url).send()?.error_for_status()?.bytes()?;
    let label = subpath
        .and_then(|s| s.trim_end_matches('/').rsplit('/').next())
        .unwrap_or(repo);
    extract_zip_skills(&bytes, label)?;
    let ws = paths::resolve_workspace()?;
    register_skill(&ws, label, "", "custom", None)?;
    println!("✓ Installed from GitHub: {spec}");
    Ok(())
}

fn parse_github_owner_repo(url: &str) -> Option<String> {
    let re = Regex::new(r"github\.com/([^/]+)/([^/#.]+)").ok()?;
    let cap = re.captures(url)?;
    Some(format!("{}/{}", &cap[1], &cap[2]))
}

fn install_local_path(path: &Path) -> Result<()> {
    let path = expand_tilde(path);
    if !path.exists() {
        bail!("path not found: {}", path.display());
    }
    let ws = paths::resolve_workspace()?;
    let dest_root = skills_dir(&ws);
    crate::io::create_dir_all(&dest_root)?;

    if path.join("SKILL.md").is_file() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "skill".into());
        copy_skill_dir(&path, &dest_root.join(&name))?;
        register_skill(&ws, &name, "", "custom", None)?;
        println!("✓ Installed local skill '{name}'");
        return Ok(());
    }

    let mut count = 0;
    for entry in crate::io::read_dir(&path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() && p.join("SKILL.md").is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            copy_skill_dir(&p, &dest_root.join(&name))?;
            register_skill(&ws, &name, "", "custom", None)?;
            count += 1;
        }
    }
    if count == 0 {
        bail!("no SKILL.md found under {}", path.display());
    }
    println!("✓ Installed {count} skill(s) from {}", path.display());
    Ok(())
}

fn extract_zip_skills(bytes: &[u8], fallback_name: &str) -> Result<()> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).context("open zip")?;
    let temp = std::env::temp_dir().join(format!("sf-skill-{}", std::process::id()));
    if temp.exists() {
        fs::remove_dir_all(&temp).ok();
    }
    crate::io::create_dir_all(&temp)?;
    archive.extract(&temp).context("extract zip")?;

    let mut skill_dirs = Vec::new();
    find_skill_dirs(&temp, &mut skill_dirs);

    if skill_dirs.is_empty() {
        bail!("no SKILL.md found in archive");
    }

    let ws = paths::resolve_workspace()?;
    let dest_root = skills_dir(&ws);
    crate::io::create_dir_all(&dest_root)?;

    if skill_dirs.len() == 1 && skill_dirs[0].1 == fallback_name {
        copy_skill_dir(&skill_dirs[0].0, &dest_root.join(fallback_name))?;
        return Ok(());
    }

    for (src, name) in skill_dirs {
        copy_skill_dir(&src, &dest_root.join(&name))?;
        register_skill(&ws, &name, "", "custom", None)?;
        println!("  + {name}");
    }
    Ok(())
}

fn find_skill_dirs(root: &Path, out: &mut Vec<(PathBuf, String)>) {
    if root.join("SKILL.md").is_file() {
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "skill".into());
        out.push((root.to_path_buf(), name));
        return;
    }
    let Ok(entries) = crate::io::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            find_skill_dirs(&p, out);
        }
    }
}

fn copy_skill_dir(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        fs::remove_dir_all(dest).ok();
    }
    copy_dir_recursive(src, dest)?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    crate::io::create_dir_all(dest)?;
    for entry in crate::io::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            crate::io::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn info(name: &str) -> Result<()> {
    let ws = paths::resolve_workspace()?;
    let mgr = SkillManager::new(&ws, None);
    if let Some(entry) = mgr.get_skill(name) {
        let cfg = load_skills_config(&ws);
        let enabled = is_enabled_in_config(&cfg, name) && entry.enabled;
        println!("\n  Skill: {}", entry.skill.name);
        println!("  Source: {}", entry.skill.source);
        println!("  Enabled: {enabled}");
        println!("  Path: {}", entry.skill.file_path);
        println!("  Description: {}\n", entry.skill.description);
        return Ok(());
    }
    let cfg = load_skills_config(&ws);
    if let Some(e) = cfg.get(name) {
        println!(
            "\n  Skill: {} (config only)\n  Enabled: {}\n  Description: {}\n",
            e.name, e.enabled, e.description
        );
        return Ok(());
    }
    bail!("skill '{name}' not found");
}

fn print_skill_table(entries: Vec<&SkillConfigEntry>, title: &str) {
    println!("\n  {title} ({})\n", entries.len());
    for e in entries {
        let status = if e.enabled { "on " } else { "off" };
        let desc = truncate(&e.description, 40);
        println!(
            "  {:<22} {:<6} {:<10} {}",
            e.display_name.as_deref().unwrap_or(&e.name),
            status,
            e.source,
            desc
        );
    }
    println!();
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    format!(
        "{}...",
        s.chars().take(max.saturating_sub(3)).collect::<String>()
    )
}

fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    path.to_path_buf()
}
