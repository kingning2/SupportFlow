use std::sync::Arc;

use models::{create_bot, BotType, CallWithToolsRequest, ModelsConfig, OpenAICompatibleBot};

#[test]
fn create_deepseek_bot_from_type() {
    let config = Arc::new(ModelsConfig {
        bot_type: BotType::DEEPSEEK.to_string(),
        model: Some("deepseek-v4-flash".into()),
        ..Default::default()
    });
    let bot = create_bot(BotType::Deepseek, config).expect("factory");
    let api = bot.get_api_config();
    assert_eq!(api.model, "deepseek-v4-flash");
    assert!(api.api_base.contains("deepseek"));
}

#[tokio::test]
#[ignore = "requires network and API key"]
async fn deepseek_call_with_tools_smoke() {
    let config = Arc::new(
        ModelsConfig::from_json_file("../../../../../CowAgent/config.json").unwrap_or_default(),
    );
    let bot = create_bot(BotType::Deepseek, config).expect("factory");
    let req = CallWithToolsRequest {
        messages: vec![serde_json::json!({"role": "user", "content": "say hi"})],
        stream: false,
        max_tokens: Some(16),
        ..Default::default()
    };
    let _ = bot.call_with_tools(req).await;
}
