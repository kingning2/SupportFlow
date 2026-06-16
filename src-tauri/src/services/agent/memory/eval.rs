//! RAG 检索评测：Recall@K / MRR@K（hybrid vs rerank 对比）。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;

use super::config::MemoryConfig;
use super::manager::DbMemoryManager;
use super::rerank::{create_rerank_provider, LexicalRerankProvider, RerankProvider};
use super::storage::MemoryStorage;
use crate::config::ModelsConfig;
use crate::services::agent::{MemoryManager, MemorySearchHit};

/// 单条标注样例。
#[derive(Debug, Clone, Deserialize)]
pub struct RagEvalCase {
    pub id: String,
    pub query: String,
    pub relevant_paths: Vec<String>,
}

/// 评测集（`tests/fixtures/rag-eval/cases.json`）。
#[derive(Debug, Clone, Deserialize)]
pub struct RagEvalSuite {
    #[serde(default = "default_k")]
    pub k: usize,
    pub cases: Vec<RagEvalCase>,
}

fn default_k() -> usize {
    5
}

/// 聚合指标。
#[derive(Debug, Clone, Copy)]
pub struct RagEvalMetrics {
    pub recall_at_k: f64,
    pub mrr_at_k: f64,
    pub hit_cases: usize,
    pub total_cases: usize,
}

impl RagEvalMetrics {
    pub fn format_row(&self, label: &str) -> String {
        format!(
            "| {label:<12} | {:>6.1}% | {:>6.3} | {}/{} |",
            self.recall_at_k * 100.0,
            self.mrr_at_k,
            self.hit_cases,
            self.total_cases
        )
    }
}

/// 一次评测运行结果（hybrid / rerank）。
#[derive(Debug, Clone)]
pub struct RagEvalRun {
    pub label: String,
    pub metrics: RagEvalMetrics,
}

/// 从 fixtures 目录加载 `cases.json`。
pub fn load_suite(fixtures_dir: &Path) -> Result<RagEvalSuite, String> {
    let path = fixtures_dir.join("cases.json");
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

/// 解析 fixtures 目录下的 `workspace/` 路径。
pub fn fixture_workspace(fixtures_dir: &Path) -> PathBuf {
    fixtures_dir.join("workspace")
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn hit_matches(hit: &MemorySearchHit, relevant_paths: &[String]) -> bool {
    let hit_path = normalize_path(&hit.path);
    relevant_paths
        .iter()
        .any(|p| hit_path.ends_with(&normalize_path(p)) || hit_path == normalize_path(p))
}

fn reciprocal_rank_at_k(hits: &[MemorySearchHit], relevant_paths: &[String], k: usize) -> f64 {
    for (idx, hit) in hits.iter().take(k).enumerate() {
        if hit_matches(hit, relevant_paths) {
            return 1.0 / (idx + 1) as f64;
        }
    }
    0.0
}

/// 对单个 manager 运行评测集。
pub async fn evaluate_manager(
    manager: &dyn MemoryManager,
    suite: &RagEvalSuite,
    min_score: f64,
) -> Result<RagEvalMetrics, String> {
    let k = suite.k.max(1);
    let total = suite.cases.len();
    if total == 0 {
        return Ok(RagEvalMetrics {
            recall_at_k: 0.0,
            mrr_at_k: 0.0,
            hit_cases: 0,
            total_cases: 0,
        });
    }

    let mut recall_sum = 0.0_f64;
    let mut mrr_sum = 0.0_f64;
    let mut hit_cases = 0_usize;

    for case in &suite.cases {
        let hits = manager.search(&case.query, None, k, min_score).await?;
        let rr = reciprocal_rank_at_k(&hits, &case.relevant_paths, k);
        mrr_sum += rr;
        if rr > 0.0 {
            recall_sum += 1.0;
            hit_cases += 1;
        }
    }

    Ok(RagEvalMetrics {
        recall_at_k: recall_sum / total as f64,
        mrr_at_k: mrr_sum / total as f64,
        hit_cases,
        total_cases: total,
    })
}

/// 构建 workspace memory manager（可选 rerank）。
pub fn build_eval_manager(
    workspace: &Path,
    models_config: &ModelsConfig,
    rerank: Option<Arc<dyn RerankProvider>>,
) -> Result<DbMemoryManager, String> {
    let mut mem_config = MemoryConfig::new(workspace);
    mem_config.enable_knowledge = true;
    mem_config.embedding_provider = models_config.embedding_provider.clone();
    mem_config.embedding_model = models_config.embedding_model.clone();
    mem_config.embedding_dimensions = models_config.embedding_dimensions;
    mem_config.rerank_provider = models_config.rerank_provider.clone();
    mem_config.rerank_model = models_config.rerank_model.clone();
    mem_config.sync_on_search = false;

    let storage = Arc::new(MemoryStorage::open(&mem_config.db_path())?);
    let embedding = super::embedding::create_embedding_provider(models_config)?;
    Ok(DbMemoryManager::new(mem_config, storage, embedding, rerank))
}

/// 运行 hybrid vs rerank 对比评测。
pub async fn run_comparison(
    workspace: &Path,
    models_config: &ModelsConfig,
    suite: &RagEvalSuite,
    min_score: f64,
) -> Result<Vec<RagEvalRun>, String> {
    let hybrid_mgr = build_eval_manager(workspace, models_config, None)?;
    hybrid_mgr.sync_index().await?;

    let hybrid_metrics = evaluate_manager(&hybrid_mgr, suite, min_score).await?;

    let rerank: Arc<dyn RerankProvider> = if let Some(p) = create_rerank_provider(models_config)? {
        p
    } else {
        Arc::new(LexicalRerankProvider)
    };

    let rerank_mgr = build_eval_manager(workspace, models_config, Some(rerank))?;
    rerank_mgr.sync_index().await?;

    let rerank_metrics = evaluate_manager(&rerank_mgr, suite, min_score).await?;

    Ok(vec![
        RagEvalRun {
            label: "hybrid".into(),
            metrics: hybrid_metrics,
        },
        RagEvalRun {
            label: "+rerank".into(),
            metrics: rerank_metrics,
        },
    ])
}

/// 打印对比表到 stdout。
pub fn print_comparison_table(runs: &[RagEvalRun], k: usize) {
    println!("\nRAG eval @ K={k}");
    println!("| mode         | Recall | MRR    | hits |");
    println!("|--------------|--------|--------|------|");
    for run in runs {
        println!("{}", run.metrics.format_row(&run.label));
    }
    println!();
}
