use std::path::{Path, PathBuf};

use tauri_app_lib::services::agent::knowledge::KnowledgeService;
use anyhow::{Context, Result};

use crate::paths;
use crate::runtime::CliRuntime;

#[derive(clap::Subcommand)]
pub enum KnowledgeCommand {
    /// Show knowledge base tree
    List,
    /// Copy file(s) into knowledge/<category>/
    Upload {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[arg(long, default_value = "uploads")]
        category: String,
    },
}

pub fn run(command: Option<KnowledgeCommand>) -> Result<()> {
    match command {
        None => println!("{}", stats()?),
        Some(KnowledgeCommand::List) => println!("{}", tree()?),
        Some(KnowledgeCommand::Upload { paths, category }) => upload(&paths, &category)?,
    }
    Ok(())
}

fn workspace_knowledge() -> Result<PathBuf> {
    let ws = paths::resolve_workspace()?;
    Ok(paths::knowledge_dir(&ws))
}

fn stats() -> Result<String> {
    let rt = CliRuntime::load()?;
    let enabled = rt.config.knowledge.unwrap_or(true);
    let svc = KnowledgeService::new(&rt.workspace);
    let tree = svc.list_tree(enabled);
    if !svc.knowledge_dir().is_dir() {
        return Ok("Knowledge base directory not found.".into());
    }
    let mut cat_count: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    fn count_nodes(
        nodes: &[tauri_app_lib::services::agent::knowledge::KnowledgeTreeNode],
        out: &mut std::collections::BTreeMap<String, usize>,
    ) {
        for n in nodes {
            if !n.files.is_empty() {
                *out.entry(n.dir.clone()).or_default() += n.files.len();
            }
            count_nodes(&n.children, out);
        }
    }
    count_nodes(&tree.tree, &mut cat_count);
    let status = if enabled { "enabled" } else { "disabled" };
    let mut lines = vec![
        String::new(),
        format!("  Knowledge Base  [{status}]"),
        String::new(),
        format!("  Pages:  {}", tree.stats.pages),
        format!("  Size:   {:.1} KB", tree.stats.size as f64 / 1024.0),
        String::new(),
    ];
    if !cat_count.is_empty() {
        lines.push("  Categories:".into());
        for (cat, n) in &cat_count {
            lines.push(format!("    {cat}/  ({n} pages)"));
        }
        lines.push(String::new());
    }
    lines.push(format!("  Path: {}", svc.knowledge_dir().display()));
    lines.push(String::new());
    Ok(lines.join("\n"))
}

fn tree() -> Result<String> {
    let dir = workspace_knowledge()?;
    if !dir.is_dir() {
        return Ok("Knowledge base directory not found.".into());
    }
    let mut lines = vec![format!("\n  {}\n", dir.display())];
    append_tree(&dir, &dir, "  ", &mut lines)?;
    lines.push(String::new());
    Ok(lines.join("\n"))
}

fn append_tree(root: &Path, current: &Path, indent: &str, lines: &mut Vec<String>) -> Result<()> {
    let mut entries: Vec<_> = fs_io::read_dir(current)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let s = n.to_string_lossy();
            !s.starts_with('.')
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_last = i + 1 == entries.len();
        let branch = if is_last { "└── " } else { "├── " };
        if path.is_dir() {
            lines.push(format!("{indent}{branch}{name}/"));
            let sub_indent = format!("{indent}{}   ", if is_last { " " } else { "│" });
            append_tree(root, &path, &sub_indent, lines)?;
        } else if name.ends_with(".md") {
            lines.push(format!("{indent}{branch}{name}"));
        }
    }
    Ok(())
}

fn upload(paths: &[PathBuf], category: &str) -> Result<()> {
    let rt = CliRuntime::load()?;
    let enabled = rt.config.knowledge.unwrap_or(true);
    let svc = KnowledgeService::new(&rt.workspace);
    let mut files = Vec::new();
    for src in paths {
        let name = src
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
            .context("upload path has no filename")?;
        let data = fs_io::read(src).with_context(|| format!("read {}", src.display()))?;
        files.push((name, data));
    }
    let result = tokio::runtime::Runtime::new()
        .context("tokio runtime")?
        .block_on(svc.ingest_upload(files, category, true, enabled, rt.config.as_ref()))
        .map_err(|e| anyhow::anyhow!(e))?;

    for item in &result.results {
        println!("  + {}", item.path);
    }
    for err in &result.errors {
        eprintln!("  ! {}: {}", err.file, err.message);
    }
    if result.memory_synced {
        println!("  Memory index updated.");
    } else if result.count > 0 {
        println!(
            "  Memory index sync skipped (keyword search still works after next agent start)."
        );
    }
    println!();
    Ok(())
}
