//! `bridge/agent_event_handler.py` — stream events to channels (logging + optional callback).

use std::sync::Arc;

use agent::AgentEvent;
use models::Context;
use tracing::debug;

const WEIXIN_THINKING_INSTANT_MAX: usize = 7;

/// Handles agent stream events; mirrors Python `AgentEventHandler`.
pub struct AgentEventHandler {
    context: Option<Context>,
    original: Option<agent::AgentEventCallback>,
    is_weixin: bool,
    thinking_sent_count: usize,
    merged_buf: Vec<String>,
    current_content: String,
}

impl AgentEventHandler {
    pub fn new(
        context: Option<Context>,
        original: Option<agent::AgentEventCallback>,
    ) -> Self {
        let is_weixin = context
            .as_ref()
            .and_then(|c| c.get("channel_type"))
            .is_some_and(|ct| ct == "weixin" || ct == "weixinmp");
        Self {
            context,
            original,
            is_weixin,
            thinking_sent_count: 0,
            merged_buf: Vec::new(),
            current_content: String::new(),
        }
    }

    pub fn handle(&mut self, event: &AgentEvent) {
        match event.event_type.as_str() {
            "turn_start" => {
                self.current_content.clear();
            }
            "message_update" => {
                if let Some(delta) = event.data.get("delta").and_then(|v| v.as_str()) {
                    self.current_content.push_str(delta);
                }
            }
            "message_end" => {
                let has_tools = event
                    .data
                    .get("tool_calls")
                    .and_then(|v| v.as_array())
                    .is_some_and(|a| !a.is_empty());
                if has_tools {
                    let snippet = self.current_content.trim().to_string();
                    if !snippet.is_empty() {
                        self.send_intermediate(&snippet);
                    }
                } else {
                    self.flush_merged();
                }
                self.current_content.clear();
            }
            "agent_end" => {
                self.flush_merged();
            }
            _ => {}
        }

        if let Some(cb) = &self.original {
            cb(event.clone());
        }
    }

    fn send_intermediate(&mut self, message: &str) {
        if self
            .context
            .as_ref()
            .and_then(|c| c.get("on_event"))
            .is_some()
        {
            return;
        }
        if !self.is_weixin {
            debug!("[AgentEventHandler] intermediate: {}", truncate(message, 200));
            return;
        }
        if self.thinking_sent_count < WEIXIN_THINKING_INSTANT_MAX {
            debug!("[AgentEventHandler] weixin thinking: {}", truncate(message, 200));
            self.thinking_sent_count += 1;
            return;
        }
        self.merged_buf.push(message.to_string());
    }

    fn flush_merged(&mut self) {
        if self.merged_buf.is_empty() {
            return;
        }
        let merged = self.merged_buf.join("\n\n");
        let count = self.merged_buf.len();
        self.merged_buf.clear();
        debug!(
            "[AgentEventHandler] flush {count} merged thinking msgs, len={}",
            merged.len()
        );
        self.thinking_sent_count += 1;
        let _ = merged;
    }

    pub fn log_summary(&self) {}
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max).collect::<String>())
    }
}

pub fn make_event_callback(
    handler: Arc<std::sync::Mutex<AgentEventHandler>>,
) -> agent::AgentEventCallback {
    Arc::new(move |ev| {
        if let Ok(mut h) = handler.lock() {
            h.handle(&ev);
        }
    })
}
