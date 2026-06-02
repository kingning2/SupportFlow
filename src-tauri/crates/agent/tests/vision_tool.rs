use agent::{AgentTool, VisionTool};
use models::ModelsConfig;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn vision_requires_image_and_question() {
    let config = Arc::new(ModelsConfig::default());
    let tool = VisionTool::new(config);
    let r1 = tool.execute(json!({ "question": "what?" })).await;
    assert_eq!(r1.status, "error");
    let r2 = tool.execute(json!({ "image": "x.png" })).await;
    assert_eq!(r2.status, "error");
}
