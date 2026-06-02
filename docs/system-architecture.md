# SupportFlow 当前系统架构（2026-06）

本文描述当前仓库里 **Tauri + Next.js + Rust Agent + Python Channel Sidecar** 的运行架构，重点覆盖：

- 前端控制台到 Tauri IPC 的调用链
- Rust Agent 运行时与流式事件
- Python 渠道 sidecar 的职责边界
- 知识上传（文件 -> Markdown -> 切块）的实际数据流

---

## 1) 总体分层

```text
+----------------------------------------------------------------------------------+
|                               Desktop App (Tauri)                               |
|                                                                                  |
|  +-------------------------+                 +--------------------------------+  |
|  | Next.js Frontend        |  invoke/event   | Rust App Core                  |  |
|  | src/app + agent-console | <-------------> | src-tauri/src/lib.rs           |  |
|  +-------------------------+                 +--------------------------------+  |
|                                                      |                           |
|                                                      v                           |
|                                           +--------------------------+           |
|                                           | AgentRuntime (context/)  |           |
|                                           | - session/config/state   |           |
|                                           | - stream emit            |           |
|                                           | - sidecar orchestration  |           |
|                                           +--------------------------+           |
|                                              |                      |            |
|                                              v                      v            |
|                                  +--------------------+    +------------------+ |
|                                  | crates/agent       |    | Python Sidecar   | |
|                                  | (in-process agent) |    | channel_agent/channel | |
|                                  +--------------------+    +------------------+ |
|                                              |                      |            |
|                                              v                      v            |
|                                  memory/knowledge/mcp/tools   external channels  |
+----------------------------------------------------------------------------------+
```

---

## 2) 前端 -> Rust IPC -> 流式回传

### 关键入口

- 前端命令封装：`src/cmd/agent.ts`
- Tauri command：`src-tauri/src/cmd/agent.rs`
- Runtime 核心：`src-tauri/src/context/agent_runtime.rs`
- 事件名：`AGENT_STREAM_CHUNK`、`AGENT_RUN_FINISHED`

### 对话链路（ASCII 时序）

```text
User
  |
  v
Frontend ChatView/use-agent-chat
  |  invoke(agent_send_message)
  v
Tauri cmd::agent_send_message
  |
  v
AgentRuntime::send_message
  |
  +--> spawn run_agent_message (tokio task)
         |
         v
       crates/agent::Agent::run_stream(...)
         |
         +--> emit AGENT_STREAM_CHUNK (delta/reasoning/tool_start/tool_end/done)
         |
         +--> emit AGENT_RUN_FINISHED (success/error)
```

说明：

- 前端通过 command 发起请求，通过 event 接收增量输出（标准 Command + Event 分工）。
- `AgentRuntime` 负责 session、provider 校验、取消请求、流式事件映射。

---

## 3) Rust Agent 子系统（`src-tauri/crates/agent`）

当前已经把核心 Agent 运行能力放在 Rust 侧（进程内）：

- prompt/skills/mcp loader
- tool manager 与工具集合（read/write/edit/bash/web_fetch/web_search/...）
- memory（SQLite + embedding + chunker）
- knowledge（文档解析、上传写入、索引）

### 逻辑边界

```text
AgentRuntime (src-tauri/context)
  -> BridgeRuntime (crates/bridge)
     -> Agent (crates/agent)
        -> models crate (provider/http client)
        -> tools/*
        -> memory/*
        -> knowledge/*
```

---

## 4) Python Channel Sidecar（渠道子系统）

Rust 主进程不会直接实现每个渠道协议，而是通过 sidecar 管理：

- sidecar 启动与保活：`src-tauri/src/context/channel_python_sidecar.rs`
- Python 入口：`src-tauri/channel_agent/channel/stdio_server.py`
- 双向通信：stdio NDJSON RPC

### 双向 RPC 关系图

```text
Rust (ChannelPythonSidecar)                          Python (channel sidecar)
-------------------------                        -------------------------
rpc("channels.list")      --------------------->  handle_rust_request
rpc("channels.action")    --------------------->  connect/disconnect/save
rpc("console.api")        --------------------->  QR/login/register APIs
rpc("channels.autostart") --------------------->  start configured channels

Python request "agent.reply" ------------------>  Rust AgentRuntime::channel_reply
Python request "channel.process" ------------->  Rust channel_runtime
Python request "channel.decorate_text" ------->  Rust channel_runtime
```

要点：

- **渠道生命周期和平台依赖在 Python**（尤其 wework/ntwork 这类能力）。
- **LLM 生成与主要 Agent 能力在 Rust**，sidecar 可回调 Rust 进行回复。

---

## 5) 知识上传链路（当前实现）

入口：

- command：`agent_upload_knowledge`
- runtime：`AgentRuntime::upload_knowledge_files`
- service：`crates/agent/src/knowledge/service.rs`
- ingest：`crates/agent/src/knowledge/ingest.rs`
- parser：`crates/agent/src/knowledge/document_parser.rs`

### 文件处理流程（含 MarkItDown）

```text
upload file bytes
    |
    v
knowledge/_sources/<uuid>_<filename>   (归档原文件)
    |
    v
parse_document_file(...)
    |
    +--> 优先 MarkItDown (knowledge/markitdown.rs -> resources/markitdown_convert.py)
    |        |
    |        v
    |      Markdown text
    |
    +--> 失败时回退 legacy parser (pdf/docx/xlsx/ppt/text)
    |
    v
truncate_head (行数/字节限制)
    |
    v
write knowledge/<category>/<slug>.md
    |
    +--> append knowledge/index.md
    +--> append knowledge/log.md
    +--> trigger memory sync (chunk + embedding + index)
```

这意味着：上传文件最终统一为 Markdown 语义文本，再进入 memory 切块与检索。

---

## 6) 配置与工作区

- **配置源（单一真相）**：`src-tauri/resources/config.json`（或 template）
- **工作区目录**：`SUPPORT_FLOW_WORKSPACE` 或 app data 目录
- 启动时会把资源配置镜像到工作区 `config.json`，方便工具读取

相关逻辑在：`resolve_agent_dirs()`（`agent_runtime.rs`）。

---

## 7) 当前架构特征（总结）

```text
UI/交互层        : Next.js + agent-console
桌面宿主层        : Tauri commands/events
智能体核心层      : Rust crates/agent + crates/models + crates/bridge
渠道协议层        : Python sidecar (channel_agent/channel)
知识记忆层        : knowledge/*.md + memory SQLite/vector
扩展层            : MCP tools + custom skills + scheduler/tasks
```

一句话概括：

**现在是“Rust 主脑 + Python 渠道外设”的双栈架构**：主链路（Agent、知识、记忆、工具、流式）在 Rust，渠道生态与平台耦合能力通过 Python sidecar 保持兼容。
