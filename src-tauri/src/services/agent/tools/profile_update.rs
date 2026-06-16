//! `profile_update` — merge traits for the current session user.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::services::agent::profile::{ProfileScope, ProfileStore, SharedProfileScope};
use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};

pub struct ProfileUpdateTool {
    store: Arc<ProfileStore>,
    scope: SharedProfileScope,
}

impl ProfileUpdateTool {
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
impl AgentTool for ProfileUpdateTool {
    fn name(&self) -> &str {
        "profile_update"
    }

    fn description(&self) -> &str {
        "更新当前会话关联用户的画像 traits；传入 null 值可删除对应键。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "traits": {
                    "type": "object",
                    "description": "Key-value traits to merge into the user profile"
                }
            },
            "required": ["traits"]
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

        let patch = match params.get("traits").and_then(|v| v.as_object()) {
            Some(m) => m.clone(),
            None => return ToolRunResult::error("Error: traits object is required"),
        };

        let merged = match self.store.update_traits(user_id, channel, patch) {
            Ok(m) => m,
            Err(e) => return ToolRunResult::error(format!("Error updating profile: {e}")),
        };

        ToolRunResult::success(json!({
            "traits": merged,
            "user_id": user_id,
            "channel": channel,
            "status": "updated"
        }))
    }
}
