//! Workflow 执行器单元测试（不调用 LLM）。

use serde_json::json;
use tauri_app_lib::services::workflow::{
    demo_linear_definition, resolve_next_node_for_test, seed_context_input,
};

#[test]
fn demo_linear_has_three_steps() {
    let def = demo_linear_definition();
    assert_eq!(def.nodes.len(), 3);
    assert_eq!(def.transitions.len(), 2);
    assert_eq!(def.entry_node_id, "step-1");
}

#[test]
fn demo_linear_resolve_next_chain() {
    let def = demo_linear_definition();
    let ctx = seed_context_input(&json!("hello"));
    assert_eq!(
        resolve_next_node_for_test(&def, "step-1", &ctx).as_deref(),
        Some("step-2")
    );
    assert_eq!(
        resolve_next_node_for_test(&def, "step-2", &ctx).as_deref(),
        Some("step-3")
    );
    assert_eq!(resolve_next_node_for_test(&def, "step-3", &ctx), None);
}
