//! `agent/tools/env_config/env_config.py`

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::info;

use crate::tools::base_tool::{AgentTool, ToolRunResult};
use crate::tools::env_config::dotenv_store::{
    delete_process_env, ensure_env_file, env_file_path, mask_value, read_env_file,
    reload_process_env, write_env_file,
};

const API_KEY_REGISTRY: &[(&str, &str)] = &[
    (
        "OPENAI_API_KEY",
        "OpenAI API 密钥 (用于GPT模型、Embedding模型)",
    ),
    ("GEMINI_API_KEY", "Google Gemini API 密钥"),
    ("CLAUDE_API_KEY", "Claude API 密钥 (用于Claude模型)"),
    (
        "LINKAI_API_KEY",
        "LinkAI智能体平台 API 密钥，支持多种模型切换",
    ),
    ("BOCHA_API_KEY", "博查 AI 搜索 API 密钥"),
    ("ZHIPUAI_API_KEY", "智谱 AI API 密钥"),
    ("QIANFAN_API_KEY", "百度千帆 API 密钥"),
];

fn key_description(key: &str) -> &'static str {
    API_KEY_REGISTRY
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, d)| *d)
        .unwrap_or("未知用途的环境变量")
}

#[derive(Clone, Default)]
pub struct EnvConfigToolConfig {
    /// Override for tests; defaults to `~/.supportflow/.env`.
    pub env_path: Option<PathBuf>,
    pub on_change: Option<Arc<dyn Fn() + Send + Sync>>,
}

pub struct EnvConfigTool {
    env_path: PathBuf,
    on_change: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl EnvConfigTool {
    pub fn new(config: EnvConfigToolConfig) -> Self {
        Self {
            env_path: config.env_path.unwrap_or_else(env_file_path),
            on_change: config.on_change,
        }
    }

    fn refresh(&self) -> bool {
        reload_process_env(&self.env_path);
        if let Some(cb) = &self.on_change {
            cb();
            true
        } else {
            false
        }
    }
}

#[async_trait]
impl AgentTool for EnvConfigTool {
    fn name(&self) -> &str {
        "env_config"
    }

    fn description(&self) -> &str {
        "Manage API keys and skill configurations securely. \
         Actions: 'set' (add/update key), 'get' (view specific key), 'list' (show all configured keys), 'delete' (remove key). \
         Values are automatically masked for security. Changes take effect immediately via hot reload."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Action to perform: 'set', 'get', 'list', 'delete'",
                    "enum": ["set", "get", "list", "delete"]
                },
                "key": {
                    "type": "string",
                    "description": "Environment variable key name (e.g. OPENAI_API_KEY, BOCHA_API_KEY)"
                },
                "value": {
                    "type": "string",
                    "description": "Value to set for the environment variable (for 'set' action)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        if let Err(e) = ensure_env_file(&self.env_path) {
            return ToolRunResult::error(format!("EnvConfig tool error: {e}"));
        }

        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        let key = params.get("key").and_then(|v| v.as_str()).map(str::trim);
        let value = params.get("value").and_then(|v| v.as_str()).map(str::trim);

        match action {
            "set" => {
                let Some(key) = key.filter(|k| !k.is_empty()) else {
                    return ToolRunResult::error(
                        "Error: 'key' and 'value' are required for 'set' action.",
                    );
                };
                let Some(value) = value.filter(|v| !v.is_empty()) else {
                    return ToolRunResult::error(
                        "Error: 'key' and 'value' are required for 'set' action.",
                    );
                };

                let mut env_vars = read_env_file(&self.env_path);
                env_vars.insert(key.to_string(), value.to_string());
                if let Err(e) = write_env_file(&self.env_path, &env_vars) {
                    return ToolRunResult::error(format!("EnvConfig tool error: {e}"));
                }
                std::env::set_var(key, value);
                info!(key, value = %mask_value(value), "EnvConfig set");

                let refreshed = self.refresh();
                let mut result = json!({
                    "message": format!("Successfully set {key}"),
                    "key": key,
                    "value": mask_value(value),
                });
                result["note"] = json!(if refreshed {
                    "✅ Skills refreshed automatically - changes are now active"
                } else {
                    "⚠️ Skills not refreshed - restart agent to load new skills"
                });
                ToolRunResult::success(result)
            }
            "get" => {
                let Some(key) = key.filter(|k| !k.is_empty()) else {
                    return ToolRunResult::error("Error: 'key' is required for 'get' action.");
                };
                let env_vars = read_env_file(&self.env_path);
                let value = env_vars
                    .get(key)
                    .cloned()
                    .or_else(|| std::env::var(key).ok());
                let description = key_description(key);
                if let Some(v) = value {
                    info!(key, value = %mask_value(&v), "EnvConfig get");
                    ToolRunResult::success(json!({
                        "key": key,
                        "value": mask_value(&v),
                        "description": description,
                        "exists": true,
                        "note": format!("Value is masked for security. In bash, use ${key} directly — it is auto-injected.")
                    }))
                } else {
                    ToolRunResult::success(json!({
                        "key": key,
                        "description": description,
                        "exists": false,
                        "message": format!("Environment variable '{key}' is not set")
                    }))
                }
            }
            "list" => {
                let env_vars = read_env_file(&self.env_path);
                let mut variables = serde_json::Map::new();
                for (k, v) in &env_vars {
                    variables.insert(
                        k.clone(),
                        json!({
                            "value": mask_value(v),
                            "description": key_description(k)
                        }),
                    );
                }
                info!(count = env_vars.len(), "EnvConfig list");
                if env_vars.is_empty() {
                    ToolRunResult::success(json!({
                        "message": "No environment variables configured",
                        "variables": {},
                        "note": "常用的 API 密钥可以通过 env_config(action='set', key='KEY_NAME', value='your-key') 来配置"
                    }))
                } else {
                    ToolRunResult::success(json!({
                        "message": format!("Found {} environment variable(s)", env_vars.len()),
                        "variables": variables
                    }))
                }
            }
            "delete" => {
                let Some(key) = key.filter(|k| !k.is_empty()) else {
                    return ToolRunResult::error("Error: 'key' is required for 'delete' action.");
                };
                let mut env_vars = read_env_file(&self.env_path);
                if !env_vars.contains_key(key) {
                    return ToolRunResult::success(json!({
                        "message": format!("Environment variable '{key}' was not set"),
                        "key": key
                    }));
                }
                env_vars.remove(key);
                if let Err(e) = write_env_file(&self.env_path, &env_vars) {
                    return ToolRunResult::error(format!("EnvConfig tool error: {e}"));
                }
                delete_process_env(key);
                info!(key, "EnvConfig delete");

                let refreshed = self.refresh();
                let mut result = json!({
                    "message": format!("Successfully deleted {key}"),
                    "key": key,
                });
                result["note"] = json!(if refreshed {
                    "✅ Skills refreshed automatically - changes are now active"
                } else {
                    "⚠️ Skills not refreshed - restart agent to apply changes"
                });
                ToolRunResult::success(result)
            }
            other => ToolRunResult::error(format!(
                "Error: Unknown action '{other}'. Use 'set', 'get', 'list', or 'delete'."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn set_get_list_delete_roundtrip() {
        let dir = TempDir::new().expect("tempdir");
        let env_path = dir.path().join(".env");
        let tool = EnvConfigTool::new(EnvConfigToolConfig {
            env_path: Some(env_path.clone()),
            on_change: None,
        });

        let set = tool
            .execute(
                json!({ "action": "set", "key": "BOCHA_API_KEY", "value": "test-key-12345678" }),
            )
            .await;
        assert_eq!(set.status, "success");

        let get = tool
            .execute(json!({ "action": "get", "key": "BOCHA_API_KEY" }))
            .await;
        assert_eq!(get.status, "success");
        assert_eq!(get.result["exists"], true);

        let list = tool.execute(json!({ "action": "list" })).await;
        assert_eq!(list.status, "success");
        assert!(list.result["variables"]["BOCHA_API_KEY"].is_object());

        let del = tool
            .execute(json!({ "action": "delete", "key": "BOCHA_API_KEY" }))
            .await;
        assert_eq!(del.status, "success");
    }
}
