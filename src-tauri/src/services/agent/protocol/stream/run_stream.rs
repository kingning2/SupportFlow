//! Multi-turn agent loop (`AgentStreamExecutor.run_stream`).

use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use crate::services::agent::protocol::stream::executor::{AgentStreamExecutor, CallLlmError};
use crate::services::agent::protocol::stream::helpers::hash_args;
use crate::services::agent::protocol::stream::tools_exec::{
    format_tool_result_content, ToolExecutionResult,
};
use crate::services::agent::protocol::AgentCancelledError;

const EXPLICIT_RESPONSE_PROMPT: &str = "请向用户说明刚才工具执行的结果或回答用户的问题。";
const LOOP_HINT: &str =
    "工具已成功执行并返回结果。请基于这些信息向用户做出回复，不要重复调用相同的工具。";
const EMPTY_FALLBACK: &str = "抱歉，我暂时无法生成回复。请尝试换一种方式描述你的需求，或稍后再试。";

#[derive(Debug, thiserror::Error)]
pub enum RunStreamError {
    #[error("agent cancelled")]
    Cancelled(#[from] AgentCancelledError),
    #[error("{0}")]
    LlmFailed(String),
    #[error("{0}")]
    Other(String),
}

impl From<CallLlmError> for RunStreamError {
    fn from(e: CallLlmError) -> Self {
        match e {
            CallLlmError::Cancelled(c) => RunStreamError::Cancelled(c),
            CallLlmError::Failed(s) => RunStreamError::LlmFailed(s),
        }
    }
}

impl AgentStreamExecutor {
    /// `AgentStreamExecutor.run_stream`
    pub async fn run_stream(&mut self, user_message: &str) -> Result<String, RunStreamError> {
        let thinking_label = if self.is_thinking_enabled() {
            " | 💭 thinking"
        } else {
            ""
        };
        info!(
            model = %self.model.model_name(),
            thinking_label,
            user = %user_message,
            "Agent run_stream start"
        );

        self.messages.push(json!({
            "role": "user",
            "content": [{"type": "text", "text": user_message}],
        }));

        self.trim_messages();
        self.validate_and_fix_messages();

        self.emit_event("agent_start", json!({}));

        let mut final_response = String::new();
        let mut turn = 0u32;
        let mut cancelled = false;
        let mut critical_abort = false;

        let run_result: Result<(), RunStreamError> = async {
            while turn < self.max_turns {
                self.check_cancelled()?;

                turn += 1;
                info!(turn, "Agent turn");
                self.emit_event("turn_start", json!({ "turn": turn }));

                let (mut assistant_msg, mut tool_calls) = self
                    .call_llm_stream(true, 0, 3, false)
                    .await?;
                final_response = assistant_msg.clone();

                if tool_calls.is_empty() {
                    if assistant_msg.is_empty() {
                        warn!("LLM returned empty response (no content, no tool calls)");
                        if turn > 1 {
                            info!("Requesting explicit response from LLM");
                            let prompt_insert_idx = self.messages.len();
                            self.messages.push(json!({
                                "role": "user",
                                "content": [{"type": "text", "text": EXPLICIT_RESPONSE_PROMPT}],
                            }));

                            let retry = self.call_llm_stream(false, 0, 3, false).await?;
                            assistant_msg = retry.0;
                            tool_calls = retry.1;
                            final_response = assistant_msg.clone();

                            if prompt_insert_idx < self.messages.len()
                                && self.messages[prompt_insert_idx].get("role")
                                    == Some(&Value::String("user".into()))
                            {
                                self.messages.remove(prompt_insert_idx);
                                debug!("Removed injected explicit-response prompt");
                            }

                            if !tool_calls.is_empty() {
                                info!("Explicit-response retry returned tool_calls, continuing");
                            } else if assistant_msg.is_empty() {
                                warn!("Still empty after explicit request");
                                final_response = EMPTY_FALLBACK.into();
                            }
                        } else {
                            final_response = EMPTY_FALLBACK.into();
                        }
                    } else {
                        let preview: String = assistant_msg.chars().take(150).collect();
                        let suffix = if assistant_msg.len() > 150 { "..." } else { "" };
                        info!(preview = %format!("{preview}{suffix}"), "Assistant text");
                    }

                    if tool_calls.is_empty() {
                        debug!("Turn complete (no tool calls)");
                        self.emit_event(
                            "turn_end",
                            json!({ "turn": turn, "has_tool_calls": false }),
                        );
                        return Ok(());
                    }
                }

                log_tool_calls(&tool_calls);

                let mut tool_result_blocks: Vec<Value> = Vec::new();

                for tc in &tool_calls {
                        self.check_cancelled()?;

                        if turn > 2 {
                            let args_hash = hash_args(&tc.arguments);
                            let repeat_count = self
                                .tool_failure_history
                                .iter()
                                .rev()
                                .take(10)
                                .filter(|(name, ahash, _)| {
                                    name == &tc.name && ahash == &args_hash
                                })
                                .count();
                            if repeat_count >= 3 {
                                warn!(
                                    tool = %tc.name,
                                    repeat_count,
                                    "Tool called repeatedly with same arguments"
                                );
                            }
                        }

                        let result = self.execute_tool(tc).await;

                        if result.status == "success" {
                            if let Some(obj) = result.result.as_object() {
                                if obj.get("type").and_then(|t| t.as_str()) == Some("file_to_send")
                                {
                                    self.files_to_send.push(result.result.clone());
                                    info!(?result.result, "File to send detected");
                                    self.emit_event("file_to_send", result.result.clone());
                                }
                            }
                        }

                        if result.status == "critical_error" {
                            error!("Critical tool error, aborting conversation");
                            final_response = result
                                .result
                                .as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| "任务执行失败".into());
                            critical_abort = true;
                            break;
                        }

                        log_tool_result(tc, &result);

                        let (content, is_error) = format_tool_result_content(&result);
                        let mut block = json!({
                            "type": "tool_result",
                            "tool_use_id": tc.id,
                            "content": content,
                        });
                        if is_error {
                            block
                                .as_object_mut()
                                .expect("block")
                                .insert("is_error".into(), Value::Bool(true));
                        }
                        tool_result_blocks.push(block);
                }

                if critical_abort {
                    break;
                }

                if !tool_result_blocks.is_empty() {
                    self.messages.push(json!({
                        "role": "user",
                        "content": tool_result_blocks,
                    }));

                    if turn >= 3 && !tool_calls.is_empty() {
                        let tool_name = &tool_calls[0].name;
                        let args_hash = hash_args(&tool_calls[0].arguments);
                        let recent_success_count = self
                            .tool_failure_history
                            .iter()
                            .rev()
                            .take(10)
                            .filter(|(name, ahash, success)| {
                                name == tool_name && ahash == &args_hash && *success
                            })
                            .count();
                        if recent_success_count >= 3 {
                            warn!(
                                tool = %tool_name,
                                recent_success_count,
                                "Potential tool loop, injecting hint"
                            );
                            self.messages.push(json!({
                                "role": "user",
                                "content": [{"type": "text", "text": LOOP_HINT}],
                            }));
                        }
                    }
                } else if !tool_calls.is_empty() {
                    warn!("Tool execution produced no results, adding emergency blocks");
                    let emergency: Vec<Value> = tool_calls
                        .iter()
                        .map(|tc| {
                            json!({
                                "type": "tool_result",
                                "tool_use_id": tc.id,
                                "content": "Error: Tool execution was interrupted",
                                "is_error": true,
                            })
                        })
                        .collect();
                    self.messages.push(json!({
                        "role": "user",
                        "content": emergency,
                    }));
                }

                self.emit_event(
                    "turn_end",
                    json!({
                        "turn": turn,
                        "has_tool_calls": true,
                        "tool_count": tool_calls.len(),
                    }),
                );
            }

            if turn >= self.max_turns {
                warn!(max_turns = self.max_turns, "Max turns reached");
                info!("Requesting summary after max steps");

                let prompt_insert_idx = self.messages.len();
                self.messages.push(json!({
                    "role": "user",
                    "content": [{
                        "type": "text",
                        "text": format!(
                            "你已经执行了{turn}个决策步骤，达到了单次运行的最大步数限制。请总结一下你目前的执行过程和结果，告诉用户当前的进展情况。不要再调用工具，直接用文字回复。"
                        ),
                    }],
                }));

                match self.call_llm_stream(false, 0, 3, false).await {
                    Ok((summary, _)) if !summary.is_empty() => {
                        final_response = summary;
                    }
                    Ok(_) => {
                        final_response = format!(
                            "我已经执行了{turn}个决策步骤，达到了单次运行的步数上限。任务可能还未完全完成，建议你将任务拆分成更小的步骤，或者换一种方式描述需求。"
                        );
                    }
                    Err(e) => {
                        warn!(?e, "Failed to get max-steps summary");
                        final_response = format!(
                            "我已经执行了{turn}个决策步骤，达到了单次运行的步数上限。任务可能还未完全完成，建议你将任务拆分成更小的步骤，或者换一种方式描述需求。"
                        );
                    }
                }

                if prompt_insert_idx < self.messages.len()
                    && self.messages[prompt_insert_idx].get("role")
                        == Some(&Value::String("user".into()))
                {
                    self.messages.remove(prompt_insert_idx);
                    debug!("Removed injected max-steps prompt");
                }
            }

            Ok(())
        }
        .await;

        match run_result {
            Err(RunStreamError::Cancelled(_)) => {
                cancelled = true;
                info!(turn, "Agent cancelled by user");
                self.handle_cancelled(&final_response);
                if final_response.trim().is_empty() {
                    final_response = "_(Cancelled)_".into();
                }
            }
            Err(RunStreamError::LlmFailed(e)) => {
                self.emit_event("error", json!({ "error": e }));
                return Err(RunStreamError::LlmFailed(e));
            }
            Err(e) => {
                error!(?e, "Agent run_stream error");
                self.emit_event("error", json!({ "error": e.to_string() }));
                return Err(e);
            }
            Ok(()) => {}
        }

        final_response = final_response.trim().to_string();
        if cancelled {
            self.emit_event(
                "agent_cancelled",
                json!({ "final_response": final_response }),
            );
        }
        info!(turn, cancelled, "Agent run_stream finished");
        self.emit_event(
            "agent_end",
            json!({ "final_response": final_response, "cancelled": cancelled }),
        );

        Ok(final_response)
    }
}

fn log_tool_calls(
    tool_calls: &[crate::services::agent::protocol::stream::executor::ParsedToolCall],
) {
    let parts: Vec<String> = tool_calls
        .iter()
        .map(|tc| {
            if let Some(obj) = tc.arguments.as_object() {
                let arg_parts: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| {
                        let v_str = v.to_string();
                        let v_display = if v_str.len() > 200 {
                            format!("{}...({} chars)", &v_str[..200], v_str.len())
                        } else {
                            v_str
                        };
                        format!("{k}={v_display}")
                    })
                    .collect();
                if arg_parts.is_empty() {
                    tc.name.clone()
                } else {
                    format!("{}({})", tc.name, arg_parts.join(", "))
                }
            } else {
                tc.name.clone()
            }
        })
        .collect();
    info!(tools = %parts.join(", "), "Tool calls");
}

fn log_tool_result(
    tc: &crate::services::agent::protocol::stream::executor::ParsedToolCall,
    result: &ToolExecutionResult,
) {
    let status_ok = result.status == "success";
    let result_str = if result.result.is_object() || result.result.is_array() {
        serde_json::to_string(&result.result).unwrap_or_default()
    } else {
        result.result.to_string()
    };
    let preview: String = result_str.chars().take(200).collect();
    let suffix = if result_str.len() > 200 { "..." } else { "" };
    info!(
        tool = %tc.name,
        success = status_ok,
        secs = result.execution_time,
        preview = %format!("{preview}{suffix}"),
        "Tool result"
    );
}
