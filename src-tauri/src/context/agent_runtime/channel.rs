//! 渠道 sidecar 入站 RPC：回复、消息处理与文本装饰。

use crate::bridge::context_from_reply_params;

use super::AgentRuntime;

impl AgentRuntime {
    pub async fn channel_reply(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "agent.reply: query required".to_string())?;
        let clear_history = params
            .get("clear_history")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let agent_default = self.config.read().await.agent_enabled();
        let use_agent = params
            .get("agent")
            .and_then(|v| v.as_bool())
            .unwrap_or(agent_default);

        let ctx = context_from_reply_params(params);
        let stack = self.bridge_stack.read().await.clone();
        let reply = stack
            .reply(query, Some(ctx), use_agent, clear_history, None)
            .await;
        Ok(reply.to_json_value())
    }

    pub async fn channel_process(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let ctx_v = params
            .get("context")
            .cloned()
            .ok_or_else(|| "channel.process: missing context".to_string())?;
        let cfg_v = params
            .get("config")
            .cloned()
            .ok_or_else(|| "channel.process: missing config".to_string())?;

        let ctx: channel_runtime::ChannelRuntimeContext =
            serde_json::from_value(ctx_v).map_err(|e| format!("channel.process context: {e}"))?;
        let cfg: channel_runtime::ChannelRuntimeConfig =
            serde_json::from_value(cfg_v).map_err(|e| format!("channel.process config: {e}"))?;
        let result = channel_runtime::process_message(&ctx, &cfg);
        let out = serde_json::to_value(result).map_err(|e| e.to_string())?;
        Ok(out)
    }

    pub async fn channel_decorate_text(
        &self,
        params: &serde_json::Value,
    ) -> Result<String, String> {
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "channel.decorate_text: missing text".to_string())?;
        let meta_v = params
            .get("meta")
            .cloned()
            .ok_or_else(|| "channel.decorate_text: missing meta".to_string())?;
        let meta: channel_runtime::ChannelRuntimeResult = serde_json::from_value(meta_v)
            .map_err(|e| format!("channel.decorate_text meta: {e}"))?;
        Ok(channel_runtime::decorate_text(text, &meta))
    }

    pub async fn channel_extract_media(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "channel.extract_media: missing text".to_string())?;
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        let items = channel_runtime::extract_media_urls(text, limit);
        serde_json::to_value(items).map_err(|e| e.to_string())
    }
}
