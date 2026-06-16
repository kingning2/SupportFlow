//! `profile_get` — read traits for the current session user.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::services::agent::profile::{ProfileScope, ProfileStore, SharedProfileScope};
use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};

pub struct ProfileGetTool {
    store: Arc<ProfileStore>,
    scope: SharedProfileScope,
}

impl ProfileGetTool {
    pub fn new(store: Arc<ProfileStore>, scope: SharedProfileScope) -> Self {
        Self { store, scope }
    }

    fn current_user(scope: &ProfileScope) -> Result<(&str, &str), String> {
        let uid = scope
            .user_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "Error: no user bound to this session".to_string())?;
        let channel = scope.channel.as_str();
        if channel.is_empty() {
            return Err("Error: channel context missing".into());
        }
        Ok((uid, channel))
    }
}

#[async_trait]
impl AgentTool for ProfileGetTool {
    fn name(&self) -> &str {
        "profile_get"
    }

    fn description(&self) -> &str {
        "读取当前会话关联用户的画像 traits（JSON 键值）。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "keys": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional trait keys to return; omit for all traits"
                }
            }
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let scope = match self.scope.lock() {
            Ok(s) => s.clone(),
            Err(_) => return ToolRunResult::error("Error: profile scope unavailable"),
        };
        let (user_id, channel) = match Self::current_user(&scope) {
            Ok(v) => v,
            Err(e) => return ToolRunResult::error(e),
        };

        let traits = match self.store.get_traits(user_id, channel) {
            Ok(t) => t,
            Err(e) => return ToolRunResult::error(format!("Error reading profile: {e}")),
        };

        if traits.is_empty() {
            return ToolRunResult::success(
                json!({ "traits": {}, "user_id": user_id, "channel": channel }),
            );
        }

        if let Some(keys) = params.get("keys").and_then(|v| v.as_array()) {
            let mut filtered = serde_json::Map::new();
            for key in keys {
                if let Some(k) = key.as_str() {
                    if let Some(v) = traits.get(k) {
                        filtered.insert(k.to_string(), v.clone());
                    }
                }
            }
            return ToolRunResult::success(json!({
                "traits": filtered,
                "user_id": user_id,
                "channel": channel
            }));
        }

        ToolRunResult::success(json!({
            "traits": traits,
            "user_id": user_id,
            "channel": channel
        }))
    }
}
