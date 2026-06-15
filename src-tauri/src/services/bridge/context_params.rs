//! Build `crate::config::Context` from sidecar `agent.reply` JSON params.

use crate::config::{Context, ContextType};

pub fn context_from_reply_params(params: &serde_json::Value) -> Context {
    let mut ctx = Context::default();
    if let Some(sid) = params.get("session_id").and_then(|v| v.as_str()) {
        ctx.set("session_id", sid);
    }
    if let Some(ct) = params.get("channel_type").and_then(|v| v.as_str()) {
        ctx.set("channel_type", ct);
    }
    if let Some(r) = params.get("receiver").and_then(|v| v.as_str()) {
        ctx.set("receiver", r);
    }
    if let Some(g) = params.get("isgroup").and_then(|v| v.as_bool()) {
        ctx.set("isgroup", if g { "1" } else { "0" });
    }
    if let Some(gn) = params.get("group_name").and_then(|v| v.as_str()) {
        ctx.set("group_name", gn);
    }
    if let Some(rid) = params.get("request_id").and_then(|v| v.as_str()) {
        ctx.set("request_id", rid);
    }
    if params
        .get("is_scheduled_task")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        ctx.set("is_scheduled_task", "1");
    }
    if let Some(q) = params.get("query").and_then(|v| v.as_str()) {
        ctx.content = Some(q.to_string());
        ctx.ty = Some(ContextType::Text);
    }
    if let Some(mt) = params.get("msg_type").and_then(|v| v.as_str()) {
        ctx.set("msg_type", mt);
    }
    if let Some(uid) = params.get("from_user_id").and_then(|v| v.as_str()) {
        ctx.set("from_user_id", uid);
    }
    ctx
}
