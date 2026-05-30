//! `models/zhipuai/zhipuai_bot.py`

use crate::session::SessionClass;
use crate::vendor_bot::vendor_bot;

vendor_bot! {
    pub struct ZhipuAiBot;
    default_model: "glm-4-flash",
    session: SessionClass::StandardChatLen,
    api_key: |c| c.zhipu_ai_api_key.clone().unwrap_or_default(),
    api_base: |c| "https://open.bigmodel.cn/api/paas/v4".to_string(),
}
