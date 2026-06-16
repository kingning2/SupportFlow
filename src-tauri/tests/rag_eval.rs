//! RAG eval baseline integration test.

use std::path::PathBuf;

use tauri_app_lib::config::ModelsConfig;
use tauri_app_lib::services::agent::memory::{
    fixture_workspace, load_suite, run_comparison, RagEvalSuite,
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .join("tests/fixtures/rag-eval")
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(from, to)?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn rag_eval_suite_has_at_least_ten_cases() {
    let fixtures = fixtures_dir();
    let suite = load_suite(&fixtures).expect("load suite");
    assert!(
        suite.cases.len() >= 10,
        "expected >= 10 cases, got {}",
        suite.cases.len()
    );
}

#[tokio::test]
async fn rag_eval_comparison_runs_on_fixtures() {
    let fixtures = fixtures_dir();
    let suite: RagEvalSuite = load_suite(&fixtures).expect("load suite");
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir_all(&fixture_workspace(&fixtures), temp.path()).expect("copy workspace");

    let config = ModelsConfig {
        bot_type: "deepseek".into(),
        rerank_provider: Some("local".into()),
        knowledge: Some(true),
        ..Default::default()
    };

    let runs = run_comparison(temp.path(), &config, &suite, 0.01)
        .await
        .expect("run comparison");
    assert_eq!(runs.len(), 2);
    assert!(runs[0].metrics.total_cases >= 10);
    assert!(runs[1].metrics.recall_at_k >= 0.0);
}
