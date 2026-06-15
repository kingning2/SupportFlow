//! WorkflowStore 持久化集成测试：创建 run → 写入 step → 重开 DB 后 load 一致。

use serde_json::json;
use tauri_app_lib::services::workflow::{
    db_path_for_workspace, NodeKind, RunStatus, StepRecord, StepStatus, WorkflowContext,
    WorkflowRun, WorkflowStore,
};

fn sample_run(run_id: &str) -> WorkflowRun {
    WorkflowRun {
        id: run_id.into(),
        definition_id: "demo-flow".into(),
        status: RunStatus::Running,
        current_node_id: Some("node-1".into()),
        context: WorkflowContext {
            vars: [("input".into(), json!({"user": "alice"}))].into(),
        },
        steps: vec![StepRecord {
            id: "step-1".into(),
            node_id: "node-1".into(),
            node_kind: NodeKind::AgentReply,
            status: StepStatus::Active,
            started_at: "2026-06-15T10:00:00Z".into(),
            finished_at: None,
            input: Some(json!({"prompt": "hello"})),
            output: None,
            error: None,
        }],
        created_at: "2026-06-15T10:00:00Z".into(),
        updated_at: "2026-06-15T10:00:00Z".into(),
        session_id: Some("sess-1".into()),
        error: None,
    }
}

#[test]
fn workflow_store_survives_reopen() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path();
    let db_path = db_path_for_workspace(workspace);

    {
        let store = WorkflowStore::open(&db_path).expect("open");
        let run = sample_run("run-001");
        store.create_run(&run).expect("create_run");
        store
            .append_event(
                "run-001",
                Some("step-1"),
                "step_started",
                &json!({"node": "node-1"}),
            )
            .expect("append_event");

        let mut completed_step = run.steps[0].clone();
        completed_step.status = StepStatus::Completed;
        completed_step.finished_at = Some("2026-06-15T10:00:05Z".into());
        completed_step.output = Some(json!({"reply": "hi"}));
        store
            .update_step("run-001", &completed_step, 0)
            .expect("update_step");
    }

    let store = WorkflowStore::open(&db_path).expect("reopen");
    let loaded = store
        .load_run("run-001")
        .expect("load_run")
        .expect("run exists");

    assert_eq!(loaded.id, "run-001");
    assert_eq!(loaded.definition_id, "demo-flow");
    assert_eq!(loaded.status, RunStatus::Running);
    assert_eq!(loaded.current_node_id.as_deref(), Some("node-1"));
    assert_eq!(
        loaded.context.vars.get("input"),
        Some(&json!({"user": "alice"}))
    );
    assert_eq!(loaded.steps.len(), 1);
    assert_eq!(loaded.steps[0].status, StepStatus::Completed);
    assert_eq!(loaded.steps[0].output, Some(json!({"reply": "hi"})));
}
