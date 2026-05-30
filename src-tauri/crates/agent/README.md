# agent

Rust port of Python `agent/`, **incremental** — one slice per PR.

## Step 1: `protocol` foundations

| Python | Rust |
|--------|------|
| `protocol/cancel.py` | `src/protocol/cancel.rs` |
| `protocol/task.py` | `src/protocol/task.rs` |
| `protocol/result.py` | `src/protocol/result.rs` |
| `protocol/models.py` | `src/protocol/models.rs` |
| `protocol/context.py` | `src/protocol/context.rs` |

## Step 2: `protocol/message_utils`

| Python | Rust |
|--------|------|
| `_repair_tool_use_adjacency` | `repair_tool_use_adjacency` |
| `sanitize_claude_messages` | `sanitize_claude_messages` |
| `drop_orphaned_tool_results_openai` | `drop_orphaned_tool_results_openai` |
| `compress_turn_to_text_only` | `compress_turn_to_text_only` |

## Step 3: `protocol/agent_stream` + LLM bridge

| Python | Rust |
|--------|------|
| `bridge/agent_bridge.AgentLLMModel` | `stream/llm_bridge::BotLlmModel` |
| `AgentStreamExecutor._call_llm_stream` | `AgentStreamExecutor::call_llm_stream` |
| `_identify_complete_turns` | `stream/turns::identify_complete_turns` |
| `_aggressive_trim_for_overflow` | `stream/turns::aggressive_trim_for_overflow` |

## Step 4: `run_stream` + trim + tool exec

| Python | Rust |
|--------|------|
| `run_stream` | `AgentStreamExecutor::run_stream` |
| `_trim_messages` | `stream/trim::trim_messages` |
| `_truncate_historical_tool_results` | `stream/trim::truncate_historical_tool_results` |
| `_execute_tool` | `stream/tools_exec::execute_tool` |
| `_check_consecutive_failures` | `tools_exec::check_consecutive_failures` |
| `tools/base_tool.py` | `tools/base_tool.rs` (`AgentTool`, `ToolRunResult`) |
| `agent.py` token helpers | `protocol/tokens.rs` |

## Step 5: `Agent` + core file tools

| Python | Rust |
|--------|------|
| `protocol/agent.py` | `protocol/agent.rs` |
| `tools/bash/bash.py` | `tools/bash/bash.rs` |
| `tools/read/read.py` | `tools/read/read.rs` |
| `tools/write/write.py` | `tools/write/write.rs` |
| `tools/edit/edit.py` | `tools/edit/edit.rs` |
| `tools/ls/ls.py` | `tools/ls/ls.rs` |
| `tools/utils/{truncate,diff}.py` | `tools/utils/{truncate,diff}.rs` |
| `tools/tool_manager.py` (core load) | `tools/tool_manager.rs` |

PDF/Office: returns same “library not installed” style errors until native extractors are wired.

## Step 6: `send` + memory tools

| Python | Rust |
|--------|------|
| `tools/send/send.py` | `tools/send/send.rs` |
| `tools/memory/memory_search.py` | `tools/memory/memory_search.rs` |
| `tools/memory/memory_get.py` | `tools/memory/memory_get.rs` |
| `MemoryManager.search` (DB/vector) | `MemoryManager` trait + `FileKeywordMemoryManager` fallback |

`SendFileUploader` trait hooks cloud upload (`copy_send_file`); default noop.

## Step 7: MCP registry + prompt + skills

| Python | Rust |
|--------|------|
| `prompt/builder.py` | `prompt/builder.rs` |
| `prompt/workspace.py` | `prompt/workspace.rs` |
| `skills/{manager,loader,formatter}.py` | `skills/` |
| `tools/mcp/mcp_tool.py` | `tools/mcp/mcp_tool.rs` |
| `ToolManager.sync_mcp_into_agent` | `McpToolRegistry::sync_into` + `AgentStreamExecutor::sync_mcp_tools` |
| `Agent.get_full_system_prompt` | `Agent::get_full_system_prompt` |

## Step 8: MCP client + background loader

| Python | Rust |
|--------|------|
| `tools/mcp/mcp_client.py` | `tools/mcp/client.rs` (stdio / SSE / streamable-http) |
| `tool_manager._load_mcp_tools*` | `tools/mcp/loader.rs` (`McpToolLoader`) |
| `mcp.json` + normalize | `tools/mcp/config.rs` |
| `McpTool.execute` | `tools/mcp/mcp_tool.rs` |

`Agent::new` starts `McpToolLoader::ensure_background_load()` when `workspace_dir` is set; `run_stream` calls `refresh_if_changed()`.

Config: `{workspace}/mcp.json` with `mcpServers` (same as SupportFlow).

## Step 9 (next)

Full `agent/memory` (SQLite + embeddings), optional tools (`web_search`, `env_config`).

## Usage

```rust
use std::sync::Arc;
use agent::{
    AgentStreamExecutor, BotLlmModel, LlmBridgeConfig, LlmModel,
};
use models::{create_bot, BotType, ModelsConfig};

let config = Arc::new(ModelsConfig::default());
let bot = create_bot(BotType::Deepseek, config).unwrap();
let model: Arc<dyn LlmModel> = Arc::new(BotLlmModel::new(
    bot,
    LlmBridgeConfig {
        model: "deepseek-v4-flash".into(),
        enable_thinking: true,
        ..Default::default()
    },
));

// High-level:
let agent = Agent::new("You are helpful", model, LlmBridgeConfig::default());
let answer = agent.run_stream("Hello", Default::default()).await?;

// Low-level executor:
let mut exec = AgentStreamExecutor::new(
    model,
    LlmBridgeConfig::default(),
    "You are a helpful assistant",
    vec![],
    50,
    None,
    None,
    30,
    None,
    None,
);

let answer = exec.run_stream("Hello").await?;
// Or low-level: exec.call_llm_stream(true, 0, 3, false).await?;
```

From Tauri: `crate::agent::*` re-exports this crate.
