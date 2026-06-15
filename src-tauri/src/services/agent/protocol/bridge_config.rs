//! rig 运行时 LLM 桥接配置。

/// 运行时 LLM 参数，由 `config.json` 与渠道上下文填充。
#[derive(Debug, Clone, Default)]
pub struct LlmBridgeConfig {
    pub model: String,
    pub enable_thinking: bool,
    pub reasoning_effort: Option<String>,
    pub channel_type: String,
    pub session_id: Option<String>,
}
