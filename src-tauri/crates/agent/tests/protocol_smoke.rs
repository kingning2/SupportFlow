use agent::{get_cancel_registry, AgentResult, LlmRequest, Task, TaskStatus};
use serde_json::json;

#[test]
fn llm_request_roundtrip() {
    let req = LlmRequest {
        messages: vec![json!({"role": "user", "content": "hi"})],
        stream: true,
        ..Default::default()
    };
    let s = serde_json::to_string(&req).unwrap();
    let back: LlmRequest = serde_json::from_str(&s).unwrap();
    assert!(back.stream);
    assert_eq!(back.messages.len(), 1);
}

#[test]
fn task_status_update() {
    let mut t = Task::new("hello");
    assert_eq!(t.status, TaskStatus::Init);
    t.update_status(TaskStatus::Processing);
    assert_eq!(t.status, TaskStatus::Processing);
    assert!(t.updated_at >= t.created_at);
}

#[test]
fn agent_result_helpers() {
    let ok = AgentResult::success("done", 3);
    assert!(!ok.is_error());
    let err = AgentResult::error("boom", 0);
    assert!(err.is_error());
}

#[test]
fn global_cancel_registry() {
    let reg = get_cancel_registry();
    let h = reg.register("integration-req", Some("sess"));
    assert!(!h.is_cancelled());
    reg.cancel_request("integration-req");
    assert!(h.is_cancelled());
    reg.unregister("integration-req");
}
