use std::sync::Arc;

use agent::{AgentStreamExecutor, AgentTool, LlmBridgeConfig, LlmModel};
use async_trait::async_trait;
use futures_util::stream;
use serde_json::{json, Value};

struct MockLlm {
    responses: Vec<(String, bool)>,
    call: std::sync::Mutex<usize>,
}

#[async_trait]
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
        let mut idx = self.call.lock().unwrap();
        let (text, with_tool) = self.responses.get(*idx).cloned().unwrap_or_default();
        *idx += 1;

        if with_tool {
            let chunks = vec![
                json!({
                    "choices": [{
                        "delta": {
                            "tool_calls": [{
                                "index": 0,
                                "id": "call_1",
                                "function": {
                                    "name": "echo",
                                    "arguments": "{\"msg\":\"hi\"}"
                                }
                            }]
                        }
                    }]
                }),
                json!({
                    "choices": [{ "finish_reason": "tool_calls", "delta": {} }]
                }),
            ];
            return Ok(Box::pin(stream::iter(chunks)));
        }

        let chunks = vec![json!({
            "choices": [{ "delta": { "content": text }, "finish_reason": "stop" }]
        })];
        Ok(Box::pin(stream::iter(chunks)))
    }
}

struct EchoTool;

#[async_trait]
impl AgentTool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "echo"
    }

    fn input_schema(&self) -> Value {
        json!({"type": "object"})
    }

    async fn execute(&self, params: Value) -> agent::ToolRunResult {
        agent::ToolRunResult::success(params)
    }
}

#[tokio::test]
async fn run_stream_text_only() {
    let model: Arc<dyn LlmModel> = Arc::new(MockLlm {
        responses: vec![("done.".into(), false)],
        call: std::sync::Mutex::new(0),
    });

    let mut exec = AgentStreamExecutor::new(
        model,
        LlmBridgeConfig::default(),
        "system",
        vec![Arc::new(EchoTool)],
        10,
        None,
        None,
        30,
        None,
        None,
    );

    let out = exec.run_stream("hello").await.expect("ok");
    assert_eq!(out, "done.");
}

#[tokio::test]
async fn run_stream_tool_then_text() {
    let model: Arc<dyn LlmModel> = Arc::new(MockLlm {
        responses: vec![(String::new(), true), ("final answer".into(), false)],
        call: std::sync::Mutex::new(0),
    });

    let mut exec = AgentStreamExecutor::new(
        model,
        LlmBridgeConfig::default(),
        "system",
        vec![Arc::new(EchoTool)],
        10,
        None,
        None,
        30,
        None,
        None,
    );

    let out = exec.run_stream("run tool").await.expect("ok");
    assert_eq!(out, "final answer");
    assert!(exec.messages.iter().any(|m| {
        m.get("content")
            .and_then(|c| c.as_array())
            .is_some_and(|blocks| {
                blocks
                    .iter()
                    .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result"))
            })
    }));
}
