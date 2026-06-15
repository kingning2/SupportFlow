//! 单轮无工具 LLM 对话（渠道非 Agent 模式、`Bridge.fetch_reply_content`）。

use std::collections::HashMap;

use crate::config::ModelsConfig;
use crate::services::agent::protocol::{LlmBridgeConfig, RunStreamError};

use super::runner::{run_rig_stream, RigRunParams};

/// 通过 rig 执行一轮无工具对话，返回 assistant 文本。
pub async fn run_simple_chat(config: &ModelsConfig, query: &str) -> Result<String, String> {
    let bridge = LlmBridgeConfig {
        model: config.model_or("deepseek-chat"),
        ..Default::default()
    };
    let output = run_rig_stream(RigRunParams {
        config,
        bridge: &bridge,
        system_prompt: String::new(),
        user_message: query.to_string(),
        messages: vec![],
        tools: HashMap::new(),
        max_steps: 1,
        on_event: None,
        cancel: None,
        mcp_registry: None,
    })
    .await
    .map_err(|e| match e {
        RunStreamError::Cancelled(_) => "agent cancelled".into(),
        RunStreamError::LlmFailed(m) => m,
        RunStreamError::Other(m) => m,
    })?;
    Ok(output.response)
}
