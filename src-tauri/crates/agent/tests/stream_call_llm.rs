use std::sync::Arc;

use agent::{
    get_cancel_registry, tools_from_schemas, AgentStreamExecutor, AgentToolSchema, LlmBridgeConfig,
    LlmModel,
};
use futures_util::stream;
use serde_json::{json, Value};

struct MockLlm {
    chunks: Vec<Value>,
    bridge: LlmBridgeConfig,
}

#[async_trait::async_trait]
impl LlmModel for MockLlm {
    fn model_name(&self) -> &str {
        "mock"
    }

    fn channel_type(&self) -> &str {
        "web"
    }

    async fn call_stream(
        &self,
        _request: &agent::LlmRequest,
    ) -> Result<agent::LlmChunkStream, agent::LlmBridgeError> {
        Ok(Box::pin(stream::iter(self.chunks.clone())))
    }
}

#[tokio::test]
async fn call_llm_stream_collects_text_and_tool_calls() {
    let chunks = vec![
        json!({
            "choices": [{
                "delta": { "content": "hello " },
            }]
        }),
        json!({
            "choices": [{
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_abc",
                        "function": { "name": "search", "arguments": "{\"q\":" }
                    }]
                },
            }]
        }),
        json!({
            "choices": [{
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "function": { "arguments": "1}" }
                    }]
                },
            }]
        }),
        json!({
            "choices": [{
                "finish_reason": "tool_calls",
                "delta": {},
            }]
        }),
    ];

    let model: Arc<dyn LlmModel> = Arc::new(MockLlm {
        chunks,
        bridge: LlmBridgeConfig {
            model: "mock".into(),
            enable_thinking: false,
            ..Default::default()
        },
    });

    let mut exec = AgentStreamExecutor::new(
        model,
        LlmBridgeConfig::default(),
        "system",
        tools_from_schemas(vec![AgentToolSchema {
            name: "search".into(),
            description: "search".into(),
            input_schema: json!({"type": "object"}),
        }]),
        10,
        None,
        Some(vec![json!({
            "role": "user",
            "content": [{"type": "text", "text": "hi"}]
        })]),
        30,
        None,
        None,
    );

    let (text, tools) = exec
        .call_llm_stream(true, 0, 3, false)
        .await
        .expect("stream ok");

    assert_eq!(text, "hello ");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "search");
    assert_eq!(tools[0].arguments["q"], 1);
    assert!(exec.messages.len() >= 2);
}

#[tokio::test]
async fn call_llm_stream_respects_cancel() {
    let chunks: Vec<Value> = (0..20)
        .map(|i| {
            json!({
                "choices": [{ "delta": { "content": format!("t{i} ") } }]
            })
        })
        .collect();

    let model: Arc<dyn LlmModel> = Arc::new(MockLlm {
        chunks,
        bridge: LlmBridgeConfig::default(),
    });

    let reg = get_cancel_registry();
    let cancel = reg.register("cancel-test", None);
    reg.cancel_request("cancel-test");

    let mut exec = AgentStreamExecutor::new(
        model,
        LlmBridgeConfig::default(),
        "system",
        vec![],
        10,
        None,
        Some(vec![json!({
            "role": "user",
            "content": [{"type": "text", "text": "hi"}]
        })]),
        30,
        Some(cancel),
        None,
    );

    let err = exec
        .call_llm_stream(true, 0, 3, false)
        .await
        .expect_err("cancelled");
    assert!(matches!(err, agent::CallLlmError::Cancelled(_)));
}
