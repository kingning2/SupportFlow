//! Rust-owned frontend channel console APIs.

use serde_json::Value;
use std::sync::Arc;
use tauri::AppHandle;

use crate::context::agent_runtime::AgentRuntime;

/// Dispatch one frontend channel console API call.
///
/// # Arguments
///
/// * `_app` - Tauri app handle (reserved for future channel APIs)
/// * `runtime` - Shared agent runtime used to reach the Python sidecar when needed
/// * `path` - Console API path such as `wework/contacts_sync`
/// * `method` - HTTP-like method string such as `GET` or `POST`
/// * `body` - JSON request body from frontend
///
/// # Returns
///
/// * `Value` - JSON payload matching the existing frontend contract
pub async fn dispatch(
    _app: &AppHandle,
    runtime: &Arc<AgentRuntime>,
    path: &str,
    method: &str,
    body: &Value,
) -> Result<Value, String> {
    let normalized_path = path.trim().trim_start_matches('/');
    let normalized_method = method.trim().to_ascii_uppercase();

    match (normalized_path, normalized_method.as_str()) {
        ("wework/contacts_sync", "POST") => wework_contacts_sync(runtime, body).await,
        _ => Err(format!(
            "unknown channel console api: {} /{}",
            normalized_method, normalized_path
        )),
    }
}

/// Trigger one manual WeCom contacts sync through the narrow Python runtime RPC.
///
/// # Arguments
///
/// * `runtime` - Shared runtime used to reach the Python sidecar
/// * `body` - JSON request body containing optional `action`
///
/// # Returns
///
/// * `Value` - Immediate accepted response while sync runs in background
async fn wework_contacts_sync(runtime: &Arc<AgentRuntime>, body: &Value) -> Result<Value, String> {
    let action = body
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("start");
    if action != "start" {
        return Err(format!("unknown action: {action}"));
    }

    runtime.request_wework_contacts_sync().await
}
