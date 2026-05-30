use futures_util::StreamExt;
use models::deepseek::agent_stream::{
    agent_error_chunk, transform_sync_response, DeepSeekAgentStream,
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
fn agent_error_chunk_shape() {
    let c = agent_error_chunk("timeout", 500);
    assert_eq!(c["error"], true);
    assert_eq!(c["status_code"], 500);
}
