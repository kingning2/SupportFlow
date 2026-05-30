use std::sync::Arc;

use models::{Bot, Context, DeepSeekBot, ModelsConfig};

#[tokio::test]
async fn deepseek_clear_memory_command() {
    let config = Arc::new(ModelsConfig::default());
    let bot = DeepSeekBot::new(config);
    let ctx = Context::text("#清除记忆", "user-1");
    let reply = bot.reply("#清除记忆", Some(&ctx)).await.expect("reply");
    assert_eq!(reply.ty, models::ReplyType::Info);
    assert_eq!(reply.content, "记忆已清除");
}
