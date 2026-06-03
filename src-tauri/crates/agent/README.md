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

Uploads / documents: **MarkItDown** → Markdown first, then `memory/chunker` (fallback: pdf-extract, calamine, OOXML). For PDF, Python helper also tries `pypdf` when MarkItDown returns empty text. Install: `pip install -r requirements-markitdown.txt` (Python 3.10+, `CHANNEL_MARKITDOWN_PYTHON`).

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

## Step 9: memory SQLite + embeddings

| Python | Rust |
|--------|------|
| `memory/storage.py` | `memory/storage.rs` |
| `memory/chunker.py` | `memory/chunker.rs` |
| `memory/manager.py` | `memory/manager.rs` |
| `memory/embedding/provider.py` | `memory/embedding.rs` |
| `create_memory_manager` | `memory/factory.rs` |

- DB path: `{workspace}/memory/long-term/index.db`
- Hybrid search: vector (cosine) + FTS5/LIKE keyword merge
- Embedding: OpenAI-compatible vendors (legacy OpenAI→LinkAI auto-pick, or explicit `embedding_provider` in config)
- Falls back to `FileKeywordMemoryManager` if SQLite init fails

## Step 10: optional tools + knowledge

| Python | Rust |
|--------|------|
| `tools/env_config/*` | `tools/env_config/*` |
| `tools/web_fetch/*` | `tools/web_fetch/*` |
| `tools/web_search/*` | `tools/web_search/*` |
| `tools/browser/*` | `tools/browser/*` (chromiumoxide + system browser) |
| `knowledge/document_parser.py` | `knowledge/document_parser.rs` + `knowledge/markitdown.rs` |
| `common/http_proxy.py` | `models/http_proxy.rs` (LLM + tools) |

## Step 11: vision + console IPC

| Python | Rust |
|--------|------|
| `tools/vision/vision.py` | `tools/vision/*` |
| `openai_compatible_bot.call_vision` | `OpenAICompatibleBot::call_vision` |
| Web 控制台 sessions/knowledge/graph | `context/workspace_console.rs` |

## Step 12 (next)

Deep Dream / memory flush；channel 全链路 Rust 化或稳定 sidecar；CLI。

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
