# bridge

Rust port of SupportFlow Agent `bridge/` for the Tauri desktop runtime.

| Python | Rust |
|--------|------|
| `bridge/bridge.py` | `bridge.rs` — bot routing, non-agent `fetch_reply_content` |
| `bridge/agent_bridge.py` | `agent_bridge.rs` — per-session `Agent`, `agent_reply` |
| `bridge/agent_initializer.py` | `agent_initializer.rs` — workspace, tools, memory, prompt |
| `bridge/agent_event_handler.py` | `agent_event_handler.rs` — stream events (weixin merge) |
| `bridge/context.py`, `reply.py` | `models::bridge::{Context, Reply}` |

Wired from `tauri-app` via `BridgeRuntime` in `context/agent_runtime.rs`.

**Not yet ported inside bridge:** voice STT/TTS, translate backends, scheduler integration, MCP hot-reload thread, daily memory flush / Deep Dream.

**Conversation persistence:** `agent::memory::conversation_store` (SQLite in `memory/long-term/index.db`), wired via `AgentInitializer` restore + `AgentBridge` append after each run.
