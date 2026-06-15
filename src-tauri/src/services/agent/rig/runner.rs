//! rig Agent 流式执行入口。

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::config::ModelsConfig;
use futures_util::StreamExt;
use rig_core::agent::{AgentBuilder, FinalResponse, MultiTurnStreamItem};
use rig_core::client::CompletionClient;
use rig_core::completion::CompletionModel;
use rig_core::message::Message;
use rig_core::providers::{anthropic, gemini, moonshot, openai};
use rig_core::streaming::{StreamedAssistantContent, StreamingChat};
use rig_core::tool::ToolDyn;
use serde_json::{json, Map, Value};
use tracing::{error, info};

use crate::services::agent::protocol::{
    AgentCancelledError, AgentEvent, AgentEventCallback, CancelHandle, LlmBridgeConfig,
    RunStreamError,
};
use crate::services::agent::tools::{AgentTool, McpToolRegistry};
use crate::services::bridge::resolve_bot_type;

use super::hooks::RigStreamHook;
use super::messages::{json_messages_to_rig, rig_messages_to_json};
use super::provider::{resolve_credentials, ProviderCredentials, ProviderFamily};
use super::tool_adapter::LegacyAgentToolAdapter;

/// rig 运行时单次执行参数。
pub struct RigRunParams<'a> {
    pub config: &'a ModelsConfig,
    pub bridge: &'a LlmBridgeConfig,
    pub system_prompt: String,
    pub user_message: String,
    pub messages: Vec<Value>,
    pub tools: HashMap<String, Arc<dyn AgentTool>>,
    pub max_steps: u32,
    pub on_event: Option<AgentEventCallback>,
    pub cancel: Option<CancelHandle>,
    pub mcp_registry: Option<Arc<McpToolRegistry>>,
}

/// rig 运行时单次执行结果。
pub struct RigRunOutput {
    pub response: String,
    pub messages: Vec<Value>,
    pub files_to_send: Vec<Value>,
    pub cancelled: bool,
}

/// 使用 rig Agent 执行一轮带工具的多轮流式对话。
pub async fn run_rig_stream(params: RigRunParams<'_>) -> Result<RigRunOutput, RunStreamError> {
    let bot_type = resolve_bot_type(params.config).map_err(|e| RunStreamError::Other(e))?;
    let creds =
        resolve_credentials(params.config, bot_type).map_err(|e| RunStreamError::Other(e))?;

    let mut tools = params.tools.clone();
    if let Some(registry) = &params.mcp_registry {
        registry.sync_into(&mut tools);
    }

    let dyn_tools: Vec<Box<dyn ToolDyn>> = tools
        .values()
        .map(|t| Box::new(LegacyAgentToolAdapter::new(t.clone())) as Box<dyn ToolDyn>)
        .collect();

    let additional_params = build_additional_params(params.bridge, params.config);
    let temperature = params.config.temperature_or(0.7) as f64;
    let max_turns = params.max_steps.max(1) as usize;
    let history = json_messages_to_rig(&params.messages);
    let files_to_send = Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
    let hook = RigStreamHook::new(
        params.on_event.clone(),
        params.cancel.clone(),
        files_to_send.clone(),
    );

    params.on_event.as_ref().map(|cb| {
        cb(AgentEvent {
            event_type: "agent_start".into(),
            timestamp: now_ts(),
            data: json!({}),
        })
    });

    let final_response = match creds.family {
        ProviderFamily::OpenAiCompat => {
            run_with_openai_compat(
                &creds,
                &params,
                dyn_tools,
                additional_params,
                temperature,
                max_turns,
                history,
                hook,
            )
            .await?
        }
        ProviderFamily::Anthropic => {
            run_with_anthropic(
                &creds,
                &params,
                dyn_tools,
                additional_params,
                temperature,
                max_turns,
                history,
                hook,
            )
            .await?
        }
        ProviderFamily::Gemini => {
            run_with_gemini(
                &creds,
                &params,
                dyn_tools,
                additional_params,
                temperature,
                max_turns,
                history,
                hook,
            )
            .await?
        }
        ProviderFamily::Moonshot => {
            run_with_moonshot(
                &creds,
                &params,
                dyn_tools,
                additional_params,
                temperature,
                max_turns,
                history,
                hook,
            )
            .await?
        }
    };

    let cancelled = params.cancel.as_ref().is_some_and(|h| h.is_cancelled());

    let response = if final_response.response().trim().is_empty() {
        "抱歉，我暂时无法生成回复。请尝试换一种方式描述你的需求，或稍后再试。".to_string()
    } else {
        final_response.response().to_string()
    };

    let updated_messages = final_response
        .history()
        .map(|h| rig_messages_to_json(h))
        .unwrap_or_else(|| {
            let mut msgs = params.messages.clone();
            msgs.push(json!({
                "role": "user",
                "content": [{"type": "text", "text": params.user_message}],
            }));
            msgs.push(json!({
                "role": "assistant",
                "content": [{"type": "text", "text": response.clone()}],
            }));
            msgs
        });

    let files = files_to_send.lock().expect("files").clone();

    Ok(RigRunOutput {
        response,
        messages: updated_messages,
        files_to_send: files,
        cancelled,
    })
}

async fn run_with_openai_compat(
    creds: &ProviderCredentials,
    params: &RigRunParams<'_>,
    dyn_tools: Vec<Box<dyn ToolDyn>>,
    additional_params: Option<Value>,
    temperature: f64,
    max_turns: usize,
    history: Vec<Message>,
    hook: RigStreamHook,
) -> Result<FinalResponse, RunStreamError> {
    let client = openai::Client::builder()
        .api_key(&creds.api_key)
        .base_url(&creds.api_base)
        .build()
        .map_err(|e| RunStreamError::Other(format!("openai client: {e}")))?
        .completions_api();
    let model = client.completion_model(&creds.model);
    let agent = build_rig_agent(
        model,
        &params.system_prompt,
        dyn_tools,
        additional_params,
        temperature,
        max_turns,
    );
    consume_rig_stream(
        agent,
        &params.user_message,
        history,
        max_turns,
        hook,
        params,
    )
    .await
}

async fn run_with_anthropic(
    creds: &ProviderCredentials,
    params: &RigRunParams<'_>,
    dyn_tools: Vec<Box<dyn ToolDyn>>,
    additional_params: Option<Value>,
    temperature: f64,
    max_turns: usize,
    history: Vec<Message>,
    hook: RigStreamHook,
) -> Result<FinalResponse, RunStreamError> {
    let client = anthropic::Client::builder()
        .api_key(&creds.api_key)
        .base_url(&creds.api_base)
        .build()
        .map_err(|e| RunStreamError::Other(format!("anthropic client: {e}")))?;
    let model = client.completion_model(&creds.model);
    let agent = build_rig_agent(
        model,
        &params.system_prompt,
        dyn_tools,
        additional_params,
        temperature,
        max_turns,
    );
    consume_rig_stream(
        agent,
        &params.user_message,
        history,
        max_turns,
        hook,
        params,
    )
    .await
}

async fn run_with_gemini(
    creds: &ProviderCredentials,
    params: &RigRunParams<'_>,
    dyn_tools: Vec<Box<dyn ToolDyn>>,
    additional_params: Option<Value>,
    temperature: f64,
    max_turns: usize,
    history: Vec<Message>,
    hook: RigStreamHook,
) -> Result<FinalResponse, RunStreamError> {
    let client = gemini::Client::builder()
        .api_key(&creds.api_key)
        .base_url(&creds.api_base)
        .build()
        .map_err(|e| RunStreamError::Other(format!("gemini client: {e}")))?;
    let model = client.completion_model(&creds.model);
    let agent = build_rig_agent(
        model,
        &params.system_prompt,
        dyn_tools,
        additional_params,
        temperature,
        max_turns,
    );
    consume_rig_stream(
        agent,
        &params.user_message,
        history,
        max_turns,
        hook,
        params,
    )
    .await
}

async fn run_with_moonshot(
    creds: &ProviderCredentials,
    params: &RigRunParams<'_>,
    dyn_tools: Vec<Box<dyn ToolDyn>>,
    additional_params: Option<Value>,
    temperature: f64,
    max_turns: usize,
    history: Vec<Message>,
    hook: RigStreamHook,
) -> Result<FinalResponse, RunStreamError> {
    let client = moonshot::Client::builder()
        .api_key(&creds.api_key)
        .base_url(&creds.api_base)
        .build()
        .map_err(|e| RunStreamError::Other(format!("moonshot client: {e}")))?;
    let model = client.completion_model(&creds.model);
    let agent = build_rig_agent(
        model,
        &params.system_prompt,
        dyn_tools,
        additional_params,
        temperature,
        max_turns,
    );
    consume_rig_stream(
        agent,
        &params.user_message,
        history,
        max_turns,
        hook,
        params,
    )
    .await
}

fn build_rig_agent<M>(
    model: M,
    system_prompt: &str,
    dyn_tools: Vec<Box<dyn ToolDyn>>,
    additional_params: Option<Value>,
    temperature: f64,
    max_turns: usize,
) -> rig_core::agent::Agent<M, ()>
where
    M: CompletionModel + 'static,
    M::StreamingResponse: rig_core::completion::GetTokenUsage,
{
    let mut builder = AgentBuilder::new(model)
        .preamble(system_prompt)
        .temperature(temperature)
        .default_max_turns(max_turns)
        .tools(dyn_tools);
    if let Some(params) = additional_params {
        builder = builder.additional_params(params);
    }
    builder.build()
}

async fn consume_rig_stream<M, P>(
    agent: rig_core::agent::Agent<M, P>,
    user_message: &str,
    history: Vec<Message>,
    max_turns: usize,
    hook: RigStreamHook,
    params: &RigRunParams<'_>,
) -> Result<FinalResponse, RunStreamError>
where
    M: CompletionModel + 'static,
    M::StreamingResponse: rig_core::completion::GetTokenUsage,
    P: rig_core::agent::PromptHook<M> + 'static,
{
    if params.cancel.as_ref().is_some_and(|h| h.is_cancelled()) {
        return Err(RunStreamError::Cancelled(AgentCancelledError));
    }

    info!(
        model = %params.bridge.model,
        tools = params.tools.len(),
        "rig run_stream start"
    );

    let mut stream = agent
        .stream_chat(Message::user(user_message), history)
        .multi_turn(max_turns)
        .with_hook(hook.clone())
        .await;

    let mut final_response = FinalResponse::empty();

    while let Some(item) = stream.next().await {
        if params.cancel.as_ref().is_some_and(|h| h.is_cancelled()) {
            return Err(RunStreamError::Cancelled(AgentCancelledError));
        }

        match item {
            Ok(MultiTurnStreamItem::StreamAssistantItem(
                StreamedAssistantContent::ReasoningDelta { reasoning, .. },
            )) => {
                if let Some(cb) = &params.on_event {
                    cb(AgentEvent {
                        event_type: "reasoning_update".into(),
                        timestamp: now_ts(),
                        data: json!({ "delta": reasoning }),
                    });
                }
            }
            Ok(MultiTurnStreamItem::FinalResponse(res)) => {
                final_response = res;
            }
            Ok(_) => {}
            Err(e) => {
                error!(error = %e, "rig stream error");
                return Err(RunStreamError::LlmFailed(e.to_string()));
            }
        }
    }

    if params.cancel.as_ref().is_some_and(|h| h.is_cancelled()) {
        if let Some(cb) = &params.on_event {
            cb(AgentEvent {
                event_type: "message_end".into(),
                timestamp: now_ts(),
                data: json!({ "cancelled": true }),
            });
        }
        return Err(RunStreamError::Cancelled(AgentCancelledError));
    }

    Ok(final_response)
}

fn build_additional_params(bridge: &LlmBridgeConfig, config: &ModelsConfig) -> Option<Value> {
    let mut extra = Map::new();
    if bridge.enable_thinking || config.enable_thinking() {
        extra.insert("thinking".into(), json!({"type": "enabled"}));
        if let Some(effort) = &bridge.reasoning_effort {
            if effort == "high" || effort == "max" {
                extra.insert("reasoning_effort".into(), json!(effort));
            }
        }
    } else {
        extra.insert("thinking".into(), json!({"type": "disabled"}));
    }
    if extra.is_empty() {
        None
    } else {
        Some(Value::Object(extra))
    }
}

fn now_ts() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
