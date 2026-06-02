//! `agent/tools/browser/browser_tool.py` — AI-facing browser tool (Rust CDP).

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use models::ModelsConfig;
use serde_json::{json, Value};

use crate::tools::base_tool::{AgentTool, ToolRunResult};
use crate::tools::browser::config::BrowserSettings;
use crate::tools::browser::service::BrowserService;

pub struct BrowserTool {
    service: Arc<BrowserService>,
    cwd: PathBuf,
}

impl BrowserTool {
    pub fn new(config: &ModelsConfig, cwd: PathBuf) -> Self {
        let settings = BrowserSettings::from_models(config);
        Self {
            service: BrowserService::new(settings),
            cwd,
        }
    }
}

#[async_trait]
impl AgentTool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "Control a browser to navigate web pages, interact with elements, and extract content. \
         Actions: navigate, snapshot, click, fill, select, scroll, screenshot, wait, back, forward, \
         get_text, press, evaluate. Uses installed Chrome/Edge in headless mode (no extra browser download). \
         Workflow: navigate → interact by ref from snapshot → snapshot to verify."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "navigate", "snapshot", "click", "fill", "select", "scroll",
                        "screenshot", "wait", "back", "forward", "get_text", "press", "evaluate"
                    ],
                    "description": "Browser action to perform"
                },
                "url": { "type": "string", "description": "URL for navigate" },
                "ref": { "type": "integer", "description": "Element ref from snapshot" },
                "selector": { "type": "string", "description": "CSS selector fallback" },
                "text": { "type": "string", "description": "Text for fill" },
                "value": { "type": "string", "description": "Option value for select" },
                "key": { "type": "string", "description": "Key for press (Enter, Tab, ...)" },
                "direction": { "type": "string", "description": "Scroll: up/down/left/right" },
                "script": { "type": "string", "description": "JS for evaluate" },
                "full_page": { "type": "boolean", "description": "Full page screenshot" },
                "timeout": { "type": "integer", "description": "Timeout in ms" }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_lowercase();

        if action.is_empty() {
            return ToolRunResult::error("Error: 'action' parameter is required");
        }

        let timeout = params.get("timeout").and_then(|v| v.as_u64()).unwrap_or(5000);
        let result = match action.as_str() {
            "navigate" => {
                let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("").trim();
                if url.is_empty() {
                    return ToolRunResult::error("Error: 'url' is required for navigate");
                }
                let url = normalize_url(url);
                self.service.navigate(&url, timeout).await
            }
            "snapshot" => self.service.snapshot().await,
            "click" => {
                let ref_id = params.get("ref").and_then(|v| v.as_u64());
                let sel = params.get("selector").and_then(|v| v.as_str());
                self.service.click(ref_id, sel, timeout).await
            }
            "fill" => {
                let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let ref_id = params.get("ref").and_then(|v| v.as_u64());
                let sel = params.get("selector").and_then(|v| v.as_str());
                self.service.fill(text, ref_id, sel, timeout).await
            }
            "select" => {
                return ToolRunResult::error(
                    "Error: select action not yet implemented in Rust browser; use click/fill or evaluate.",
                );
            }
            "scroll" => {
                let direction = params
                    .get("direction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("down");
                let amount = params.get("amount").and_then(|v| v.as_i64()).unwrap_or(500) as i32;
                self.service.scroll(direction, amount).await
            }
            "screenshot" => {
                let full = params
                    .get("full_page")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                self.service.screenshot(full, &self.cwd).await
            }
            "wait" => {
                let sel = params.get("selector").and_then(|v| v.as_str());
                self.service.wait(sel, timeout).await
            }
            "back" => self.service.back().await,
            "forward" => self.service.forward().await,
            "get_text" => {
                let sel = params.get("selector").and_then(|v| v.as_str()).unwrap_or("").trim();
                if sel.is_empty() {
                    return ToolRunResult::error("Error: 'selector' is required for get_text");
                }
                self.service.get_text(sel).await
            }
            "press" => {
                let key = params.get("key").and_then(|v| v.as_str()).unwrap_or("").trim();
                if key.is_empty() {
                    return ToolRunResult::error("Error: 'key' is required for press");
                }
                self.service.press(key).await
            }
            "evaluate" => {
                let script = params.get("script").and_then(|v| v.as_str()).unwrap_or("").trim();
                if script.is_empty() {
                    return ToolRunResult::error("Error: 'script' is required for evaluate");
                }
                self.service.evaluate(script).await
            }
            other => {
                return ToolRunResult::error(format!(
                    "Unknown action '{other}'. Valid: navigate, snapshot, click, fill, scroll, screenshot, wait, back, forward, get_text, press, evaluate"
                ));
            }
        };

        match result {
            Ok(text) => ToolRunResult::success_text(text),
            Err(msg) => ToolRunResult::error(format!("Browser error ({action}): {msg}")),
        }
    }
}

fn normalize_url(url: &str) -> String {
    if url.contains("://") || url.starts_with("about:") || url.starts_with("data:") {
        url.to_string()
    } else {
        format!("https://{url}")
    }
}
