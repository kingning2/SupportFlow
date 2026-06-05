//! Message trimming (`_trim_messages`, `_truncate_historical_tool_results`).

use serde_json::Value;
use tracing::info;

use crate::services::agent::protocol::stream::executor::AgentStreamExecutor;
use crate::services::agent::protocol::stream::turns::{
    compress_turn, identify_complete_turns, Turn,
};
use crate::services::agent::protocol::tokens::{
    context_reserve_tokens, estimate_message_tokens, estimate_turn_tokens, model_context_window,
};

const MAX_HISTORY_RESULT_CHARS: usize = 20_000;
const COMPRESS_THRESHOLD: usize = 5;

/// Truncate historical `tool_result` blocks before token trim.
pub fn truncate_historical_tool_results(messages: &mut [Value]) {
    if messages.len() < 2 {
        return;
    }

    let mut current_turn_start = messages.len();
    for i in (0..messages.len()).rev() {
        let msg = &messages[i];
        if msg.get("role") != Some(&Value::String("user".into())) {
            continue;
        }
        let content = msg.get("content").cloned().unwrap_or(Value::Null);
        if let Some(blocks) = content.as_array() {
            if blocks
                .iter()
                .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
            {
                current_turn_start = i;
                break;
            }
        } else if content.is_string() {
            current_turn_start = i;
            break;
        }
    }

    let mut truncated_count = 0u32;
    for msg in messages.iter_mut().take(current_turn_start) {
        if msg.get("role") != Some(&Value::String("user".into())) {
            continue;
        }
        let Some(blocks) = msg.get_mut("content").and_then(|c| c.as_array_mut()) else {
            continue;
        };
        for block in blocks.iter_mut() {
            if block.get("type").and_then(|t| t.as_str()) != Some("tool_result") {
                continue;
            }
            let Some(result_str) = block.get("content").and_then(|c| c.as_str()) else {
                continue;
            };
            if result_str.len() <= MAX_HISTORY_RESULT_CHARS {
                continue;
            }
            let original_len = result_str.len();
            let new_content = format!(
                "{}{}",
                &result_str[..MAX_HISTORY_RESULT_CHARS],
                format!(
                    "\n\n[Historical output truncated: {original_len} -> {MAX_HISTORY_RESULT_CHARS} chars]"
                )
            );
            block
                .as_object_mut()
                .expect("tool_result block")
                .insert("content".into(), Value::String(new_content));
            truncated_count += 1;
        }
    }

    if truncated_count > 0 {
        info!(
            count = truncated_count,
            limit = MAX_HISTORY_RESULT_CHARS,
            "Truncated historical tool results"
        );
    }
}

impl AgentStreamExecutor {
    /// `AgentStreamExecutor._trim_messages`
    pub fn trim_messages(&mut self) {
        if self.messages.is_empty() {
            return;
        }

        truncate_historical_tool_results(&mut self.messages);

        let mut turns = identify_complete_turns(&self.messages);
        if turns.is_empty() {
            return;
        }

        if turns.len() > self.max_context_turns as usize {
            let removed_count = turns.len() / 2;
            let keep_count = turns.len() - removed_count;
            let discarded_turns: Vec<Turn> = turns.drain(..removed_count).collect();

            info!(
                total = keep_count + removed_count,
                max_context_turns = self.max_context_turns,
                keep = keep_count,
                removed = removed_count,
                "Context turn limit exceeded, trimming"
            );

            let discarded_messages: Vec<Value> = discarded_turns
                .iter()
                .flat_map(|t| t.messages.clone())
                .collect();
            if !discarded_messages.is_empty() {
                if let Some(host) = &self.host {
                    host.memory_flush_on_trim(&discarded_messages, "trim", removed_count);
                }
            }
        }

        let context_window = self
            .host
            .as_ref()
            .map(|h| h.context_window_tokens())
            .unwrap_or_else(|| model_context_window(self.model.model_name()));

        let max_tokens = self
            .host
            .as_ref()
            .and_then(|h| h.max_context_tokens())
            .unwrap_or_else(|| {
                let reserve = context_reserve_tokens(context_window, None);
                context_window.saturating_sub(reserve)
            });

        let system_tokens = estimate_message_tokens(&serde_json::json!({
            "role": "system",
            "content": self.system_prompt,
        }));
        let _available_tokens = max_tokens.saturating_sub(system_tokens);

        let current_tokens: u32 = turns
            .iter()
            .map(|t| estimate_turn_tokens(&t.messages))
            .sum();

        if current_tokens + system_tokens <= max_tokens {
            let new_messages: Vec<Value> = turns.into_iter().flat_map(|t| t.messages).collect();
            let old_count = self.messages.len();
            self.messages = new_messages;
            if old_count > self.messages.len() {
                info!(
                    old_count,
                    new_count = self.messages.len(),
                    "Rebuilt message list after turn trim"
                );
            }
            return;
        }

        if turns.len() < COMPRESS_THRESHOLD {
            let compressed_turns: Vec<Turn> = turns
                .iter()
                .map(compress_turn)
                .filter(|t| !t.messages.is_empty())
                .collect();
            let new_messages: Vec<Value> = compressed_turns
                .iter()
                .flat_map(|t| t.messages.clone())
                .collect();
            let new_tokens: u32 = compressed_turns
                .iter()
                .map(|t| estimate_turn_tokens(&t.messages))
                .sum();
            let old_count = self.messages.len();
            self.messages = new_messages;
            info!(
                turns = turns.len(),
                old_count,
                new_count = self.messages.len(),
                before_tokens = current_tokens + system_tokens,
                max_tokens,
                after_tokens = new_tokens + system_tokens,
                "Compressed all turns to text-only (few turns)"
            );
            return;
        }

        let removed_count = turns.len() / 2;
        let keep_count = turns.len() - removed_count;
        let discarded_turns: Vec<Turn> = turns.drain(..removed_count).collect();
        let kept_turns: Vec<Turn> = turns;
        let kept_tokens: u32 = kept_turns
            .iter()
            .map(|t| estimate_turn_tokens(&t.messages))
            .sum();

        info!(
            before_tokens = current_tokens + system_tokens,
            max_tokens,
            keep = keep_count,
            removed = removed_count,
            "Context token limit exceeded, discarding older half of turns"
        );

        let discarded_messages: Vec<Value> = discarded_turns
            .iter()
            .flat_map(|t| t.messages.clone())
            .collect();
        if !discarded_messages.is_empty() {
            if let Some(host) = &self.host {
                host.memory_flush_on_trim(&discarded_messages, "trim", removed_count);
            }
        }

        let new_messages: Vec<Value> = kept_turns.iter().flat_map(|t| t.messages.clone()).collect();
        let old_count = self.messages.len();
        self.messages = new_messages;
        info!(
            removed_turns = removed_count,
            old_count,
            new_count = self.messages.len(),
            before_tokens = current_tokens + system_tokens,
            after_tokens = kept_tokens + system_tokens,
            "Discarded older turns for token limit"
        );
    }
}
