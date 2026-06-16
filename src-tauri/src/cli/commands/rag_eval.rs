//! `sf rag-eval` — RAG 检索 Recall@K / MRR@K 基线评测。

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::ModelsConfig;
use crate::services::agent::memory::{
    fixture_workspace, load_suite, print_comparison_table, run_comparison,
};

#[derive(Args)]
pub struct RagEvalArgs {
    /// Workspace 目录（默认：fixtures 内 `workspace/`）
    #[arg(long)]
    pub workspace: Option<PathBuf>,
    /// Fixtures 根目录（含 `cases.json` 与 `workspace/`）
    #[arg(long)]
    pub fixtures: Option<PathBuf>,
    /// 最低相关性分数阈值
    #[arg(long, default_value_t = 0.01)]
    pub min_score: f64,
}

fn default_fixtures_dir() -> Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest
        .parent()
        .context("repo root")?
        .join("tests/fixtures/rag-eval");
    if fixtures.join("cases.json").is_file() {
        Ok(fixtures)
    } else {
        anyhow::bail!(
            "fixtures not found at {}; pass --fixtures PATH",
            fixtures.display()
        )
    }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

pub async fn run(args: RagEvalArgs) -> Result<()> {
    let fixtures = args.fixtures.map(Ok).unwrap_or_else(default_fixtures_dir)?;
    let source_workspace = args
        .workspace
        .unwrap_or_else(|| fixture_workspace(&fixtures));

    if !source_workspace.is_dir() {
        anyhow::bail!("workspace not found: {}", source_workspace.display());
    }

    let temp = tempfile::tempdir().context("temp workspace")?;
    copy_dir_all(&source_workspace, temp.path())?;
    let workspace = temp.path().to_path_buf();

    let suite = load_suite(&fixtures).map_err(anyhow::Error::msg)?;

    let mut config = ModelsConfig {
        bot_type: "deepseek".into(),
        knowledge: Some(true),
        rerank_provider: Some("local".into()),
        ..Default::default()
    };
    if let Ok(rt) = crate::cli::runtime::CliRuntime::load() {
        config = (*rt.config).clone();
        if config.rerank_provider.is_none() {
            config.rerank_provider = Some("local".into());
        }
    }

    let runs = run_comparison(&workspace, &config, &suite, args.min_score)
        .await
        .map_err(anyhow::Error::msg)?;

    print_comparison_table(&runs, suite.k);
    Ok(())
}
