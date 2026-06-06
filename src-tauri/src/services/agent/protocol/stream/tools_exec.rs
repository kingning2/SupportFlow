//! Tool execution helpers (`_execute_tool`, failure tracking).

use std::time::Instant;

use serde_json::{json, Value};
use tracing::{error, info};

use crate::services::agent::protocol::stream::executor::{AgentStreamExecutor, ParsedToolCall};
use crate::services::agent::protocol::stream::helpers::hash_args;
use crate::services::agent::tools::{ToolRunResult, ToolStage};

const MAX_CURRENT_TURN_RESULT_CHARS: usize = 50_000;

#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub status: String,
    pub result: Value,
    pub execution_time: f64,
}

impl From<ToolRunResult> for ToolExecutionResult {
    fn from(r: ToolRunResult) -> Self {
        Self {
            status: r.status,
            result: r.result,
            execution_time: r.execution_time,
        }
    }
}

impl AgentStreamExecutor {
    pub(crate) fn check_consecutive_failures(
        &self,
        tool_name: &str,
        args: &Value,
    ) -> (bool, String, bool) {
        let args_hash = hash_args(args);

        let mut same_args_calls = 0u32;
        for (name, ahash, _success) in self.tool_failure_history.iter().rev() {
            if name == tool_name && *ahash == args_hash {
                same_args_calls += 1;
            } else {
                break;
            }
        }
        if same_args_calls >= 5 {
            return (
                true,
                format!(
                    "工具 '{tool_name}' 使用相同参数已被调用 {same_args_calls} 次，停止执行以防止无限循环。如果需要查看配置，结果已在之前的调用中返回。"
                ),
                false,
            );
        }

        let mut same_args_failures = 0u32;
        for (name, ahash, success) in self.tool_failure_history.iter().rev() {
            if name == tool_name && *ahash == args_hash {
                if !*success {
                    same_args_failures += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if same_args_failures >= 3 {
            return (
                true,
                format!(
                    "工具 '{tool_name}' 使用相同参数连续失败 {same_args_failures} 次，停止执行以防止无限循环"
                ),
                false,
            );
        }

        let mut same_tool_failures = 0u32;
        for (name, _ahash, success) in self.tool_failure_history.iter().rev() {
            if name == tool_name {
                if !*success {
                    same_tool_failures += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if same_tool_failures >= 8 {
            return (
                true,
                "抱歉，我没能完成这个任务。可能是我理解有误或者当前方法不太合适。\n\n建议你：\n• 换个方式描述需求试试\n• 把任务拆分成更小的步骤\n• 或者换个思路来解决".into(),
                true,
            );
        }

        if same_tool_failures >= 6 {
            return (
                true,
                format!(
                    "工具 '{tool_name}' 连续失败 {same_tool_failures} 次（使用不同参数），停止执行以防止无限循环"
                ),
                false,
            );
        }

        (false, String::new(), false)
    }

    pub(crate) fn record_tool_result(&mut self, tool_name: &str, args: &Value, success: bool) {
        let args_hash = hash_args(args);
        self.tool_failure_history
            .push((tool_name.to_string(), args_hash, success));
        if self.tool_failure_history.len() > 50 {
            let drain = self.tool_failure_history.len() - 50;
            self.tool_failure_history.drain(..drain);
        }
    }

    fn build_tool_not_found_message(&self, tool_name: &str) -> String {
        let available: Vec<&str> = self.tools.keys().map(|s| s.as_str()).collect();
        format!("Tool '{tool_name}' not found. Available tools: {available:?}")
    }

    /// `AgentStreamExecutor._execute_tool`
    pub async fn execute_tool(&mut self, tool_call: &ParsedToolCall) -> ToolExecutionResult {
        let tool_name = &tool_call.name;
        let tool_id = &tool_call.id;
        let arguments = &tool_call.arguments;

        if let Some(parse_err) = &tool_call.parse_error {
            self.record_tool_result(tool_name, arguments, false);
            return ToolExecutionResult {
                status: "error".into(),
                result: Value::String(parse_err.clone()),
                execution_time: 0.0,
            };
        }

        let (should_stop, stop_reason, is_critical) =
            self.check_consecutive_failures(tool_name, arguments);
        if should_stop {
            error!(tool = %tool_name, %stop_reason, "Tool stopped by failure guard");
            self.record_tool_result(tool_name, arguments, false);
            if is_critical {
                return ToolExecutionResult {
                    status: "critical_error".into(),
                    result: Value::String(stop_reason),
                    execution_time: 0.0,
                };
            }
            return ToolExecutionResult {
                status: "error".into(),
                result: Value::String(format!(
                    "{stop_reason}\n\n当前方法行不通，请尝试完全不同的方法或向用户询问更多信息。"
                )),
                execution_time: 0.0,
            };
        }

        self.emit_event(
            "tool_execution_start",
            json!({
                "tool_call_id": tool_id,
                "tool_name": tool_name,
                "arguments": arguments,
            }),
        );

        let result = match self.tools.get(tool_name) {
            Some(tool) => {
                if tool.stage() != ToolStage::PreProcess {
                    let msg = format!("Tool '{tool_name}' is not available for agent invocation");
                    ToolExecutionResult {
                        status: "error".into(),
                        result: Value::String(msg),
                        execution_time: 0.0,
                    }
                } else {
                    let start = Instant::now();
                    let run: ToolRunResult = tool.execute(arguments.clone()).await;
                    let mut exec_result = ToolExecutionResult::from(run);
                    exec_result.execution_time = start.elapsed().as_secs_f64();
                    exec_result
                }
            }
            None => ToolExecutionResult {
                status: "error".into(),
                result: Value::String(self.build_tool_not_found_message(tool_name)),
                execution_time: 0.0,
            },
        };

        let success = result.status == "success";
        self.record_tool_result(tool_name, arguments, success);

        self.emit_event(
            "tool_execution_end",
            json!({
                "tool_call_id": tool_id,
                "tool_name": tool_name,
                "status": result.status,
                "result": result.result,
                "execution_time": result.execution_time,
            }),
        );

        result
    }
}

/// Format tool result for Claude `tool_result` block content.
pub fn format_tool_result_content(result: &ToolExecutionResult) -> (String, bool) {
    let is_error = result.status == "error";
    let content = if is_error {
        format!(
            "Error: {}",
            result
                .result
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| result.result.to_string())
        )
    } else if result.result.is_object() || result.result.is_array() {
        serde_json::to_string(&result.result).unwrap_or_else(|_| result.result.to_string())
    } else if let Some(s) = result.result.as_str() {
        s.to_string()
    } else {
        format!("status={} result={}", result.status, result.result)
    };

    let mut content = content;
    if content.len() > MAX_CURRENT_TURN_RESULT_CHARS {
        let truncated_len = content.len();
        let suffix = format!(
            "\n\n[Output truncated: {truncated_len} chars total, showing first {MAX_CURRENT_TURN_RESULT_CHARS} chars]"
        );
        content = format!("{}{}", &content[..MAX_CURRENT_TURN_RESULT_CHARS], suffix);
        info!(
            truncated_len,
            limit = MAX_CURRENT_TURN_RESULT_CHARS,
            "Truncated current-turn tool result"
        );
    }

    (content, is_error)
}
