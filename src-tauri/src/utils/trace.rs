//! Unified trace id for log correlation across agent / workflow / channel.

/// Derive a stable trace id from the best available correlation keys.
pub fn resolve_trace_id(
    session_id: Option<&str>,
    workflow_run_id: Option<&str>,
    message_id: Option<&str>,
) -> String {
    if let Some(run) = workflow_run_id.filter(|s| !s.is_empty()) {
        return format!("wf-{run}");
    }
    if let Some(sid) = session_id.filter(|s| !s.is_empty()) {
        if let Some(mid) = message_id.filter(|s| !s.is_empty()) {
            return format!("{sid}:{mid}");
        }
        return sid.to_string();
    }
    if let Some(mid) = message_id.filter(|s| !s.is_empty()) {
        return format!("msg-{mid}");
    }
    format!("anon-{:x}", random_nonce())
}

fn random_nonce() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_workflow_run_id() {
        let id = resolve_trace_id(Some("s1"), Some("run-9"), Some("m1"));
        assert_eq!(id, "wf-run-9");
    }

    #[test]
    fn combines_session_and_message() {
        let id = resolve_trace_id(Some("sess"), None, Some("req-1"));
        assert_eq!(id, "sess:req-1");
    }
}
