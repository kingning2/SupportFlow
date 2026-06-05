use futures_util::StreamExt;
use models::deepseek::agent_stream::{
    agent_error_chunk, transform_raw_sse_chunk, transform_sync_response, DeepSeekAgentStream,
};
use serde_json::json;

#[tokio::test]
async fn agent_stream_emits_deltas_and_terminal_finish() {
    let raw = vec![
        json!({
            "choices": [{
                "delta": { "reasoning_content": "a", "content": "b" },
            }]
        }),
        json!({
            "choices": [{
                "finish_reason": "stop",
                "delta": {},
            }]
        }),
    ];
    let mut stream = DeepSeekAgentStream::from_raw_chunks(raw);

    let mut chunks = Vec::new();
    while let Some(c) = stream.next().await {
        chunks.push(c);
    }

    assert!(chunks.len() >= 3);
    assert!(chunks[0]["choices"][0]["delta"]["reasoning_content"].is_string());
    assert_eq!(chunks[1]["choices"][0]["delta"]["content"], "b");
    let last = chunks.last().unwrap();
    assert_eq!(last["choices"][0]["finish_reason"], "stop");
}

#[test]
fn sync_response_maps_stop_and_tool_use() {
    let raw = json!({
        "choices": [{
            "finish_reason": "stop",
            "message": { "content": "done" }
        }]
    });
    let out = transform_sync_response(&raw);
    assert_eq!(out["stop_reason"], "end_turn");
    assert_eq!(out["content"][0]["text"], "done");
}

#[test]
fn transform_reasoning_and_content_deltas() {
    let raw = json!({
        "choices": [{
            "delta": {
                "reasoning_content": "think",
                "content": "hi",
            }
        }]
    });
    let chunks = transform_raw_sse_chunk(&raw);
    assert_eq!(chunks.len(), 2);
    assert!(chunks[0]["choices"][0]["delta"]["reasoning_content"].is_string());
    assert_eq!(chunks[1]["choices"][0]["delta"]["content"], "hi");
}

#[test]
fn transform_sync_tool_use_stop_reason() {
    let raw = json!({
        "choices": [{
            "finish_reason": "tool_calls",
            "message": {
                "reasoning_content": "r",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "function": { "name": "search", "arguments": "{\"q\":1}" }
                }]
            }
        }]
    });
    let out = transform_sync_response(&raw);
    assert_eq!(out["stop_reason"], "tool_use");
    let blocks = out["content"].as_array().unwrap();
    assert_eq!(blocks[0]["type"], "thinking");
    assert_eq!(blocks[1]["type"], "tool_use");
    assert_eq!(blocks[1]["input"]["q"], 1);
}

#[test]
fn agent_error_chunk_shape() {
    let c = agent_error_chunk("timeout", 500);
    assert_eq!(c["error"], true);
    assert_eq!(c["status_code"], 500);
}
