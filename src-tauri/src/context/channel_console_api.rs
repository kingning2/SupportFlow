//! Rust-owned frontend channel console APIs.

use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::context::agent_runtime::AgentRuntime;
use crate::context::channel_status::ChannelStatusStore;

/// Dispatch one frontend channel console API call.
///
/// # Arguments
///
/// * `app` - Tauri app handle used to access managed stores
/// * `runtime` - Shared agent runtime used to reach the Python sidecar when needed
/// * `path` - Console API path such as `wx/qrlogin`
/// * `method` - HTTP-like method string such as `GET` or `POST`
/// * `body` - JSON request body from frontend
///
/// # Returns
///
/// * `Value` - JSON payload matching the existing frontend contract
pub async fn dispatch(
    app: &AppHandle,
    runtime: &Arc<AgentRuntime>,
    path: &str,
    method: &str,
    body: &Value,
) -> Result<Value, String> {
    let normalized_path = path.trim().trim_start_matches('/');
    let normalized_method = method.trim().to_ascii_uppercase();

    match (normalized_path, normalized_method.as_str()) {
        ("wx/qrlogin", "GET") | ("wx/qrlogin", "POST") => wx_qrlogin(app),
        ("wework/contacts_sync", "POST") => wework_contacts_sync(runtime, body).await,
        _ => Err(format!(
            "unknown channel console api: {} /{}",
            normalized_method, normalized_path
        )),
    }
}

/// Return current personal WeChat QR/login state derived from Rust status store.
///
/// # Arguments
///
/// * `app` - Tauri app handle used to read the managed status store
///
/// # Returns
///
/// * `Value` - Existing QR login response shape expected by frontend
fn wx_qrlogin(app: &AppHandle) -> Result<Value, String> {
    let store = app.state::<ChannelStatusStore>();
    let status = store.get("wx")?;

    let Some(status) = status else {
        return Ok(json!({
            "status": "success",
            "login_status": "idle",
            "message": "Start wx channel first or wait for QR",
        }));
    };

    let login_status = phase_to_wx_login_status(&status.phase);
    if login_status == "logged_in" {
        return Ok(json!({
            "status": "success",
            "qr_status": "confirmed",
            "login_status": "logged_in",
        }));
    }

    let qr_status = match login_status {
        "scanned" => "scaned",
        _ => "wait",
    };

    Ok(json!({
        "status": "success",
        "qr_status": qr_status,
        "login_status": login_status,
        "qrcode_url": status.qr_code_url,
        "qr_image": status.qr_image,
        "message": status.message,
    }))
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
/// * `Value` - Existing manual sync response shape expected by frontend
async fn wework_contacts_sync(runtime: &Arc<AgentRuntime>, body: &Value) -> Result<Value, String> {
    let action = body
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("start");
    if action != "start" {
        return Err(format!("unknown action: {action}"));
    }

    let sidecar = runtime.ensure_channel_sidecar().await?;
    sidecar.wework_sync_contacts().await
}

/// Map raw runtime phase into the frontend login status vocabulary.
///
/// # Arguments
///
/// * `phase` - Raw phase emitted by the Python sidecar
///
/// # Returns
///
/// * `&str` - Frontend login status string
fn phase_to_wx_login_status(phase: &str) -> &str {
    match phase {
        "waiting_scan" => "waiting_scan",
        "scanned" => "scanned",
        "logged_in" | "ready" => "logged_in",
        _ => "idle",
    }
}
