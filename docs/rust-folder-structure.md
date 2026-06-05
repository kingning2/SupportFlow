# Rust 文件夹结构

## 目标

Rust 拥有桌面应用编排、状态存储、配置、IPC、AI 工具链和跨端共享业务逻辑。

## 核心目录

- `src-tauri/src/cmd/`
  - Tauri command 入口
- `src-tauri/src/context/`
  - 跨 Webview 共享状态、sidecar 运行态、运行时编排
- `src-tauri/src/events/`
  - 事件名、事件发射、事件载荷
- `src-tauri/src/utils/`
  - 无全局 Store 的可复用逻辑
- `src-tauri/src/lib.rs`
  - Tauri 应用入口与 `.manage(...)`

## Rust crate 分层

**原则：`crates/` 只放基础设施与可复用库；桌面业务编排放在 `src/`。**

- `src-tauri/crates/fs_io/`
  - 全仓库唯一的带日志文件 IO（禁止再复制 `fs.rs`）
- `src-tauri/crates/channel_runtime/`
  - 渠道消息规则（纯函数，无 Store）
- `src-tauri/crates/models/`
  - LLM Provider 协议与 `config.json` 模型（非桌面编排）
- `src-tauri/src/services/agent/`
  - Agent 工具引擎（原 `crates/agent`）
- `src-tauri/src/services/bridge/`
  - Bot 路由、`AgentBridge`（原 `crates/bridge`）
- `src-tauri/crates/cli/`
  - `sf` CLI 二进制

## `src/` 业务模块

```
src/
  services/
    agent/          # 工具链（read/write/bash/MCP/…）
    bridge/         # BridgeRuntime、AgentBridge
  context/
    channel/        # 渠道 sidecar、收件箱、配置（原 channel_*.rs）
      config.rs     # connect/disconnect（勿与 channel_runtime crate 混淆）
    agent_runtime.rs
  python/           # Python 互操作（路径、MarkItDown、sidecar RPC）
  cmd/
    agent_ipc.rs    # 原 cmd/agent.rs（避免与 crate::agent 重名）
```

CLI（`sf`）通过 `tauri-app` 的 `default-features = false` 依赖 `services::*`，不链接 Tauri。

## 结构原则

1. `cmd` 只做命令入口与参数/返回值定义。
2. `context` 只做共享状态、sidecar 运行时协调、状态同步。
3. `utils` 只做无状态工具逻辑。
4. 任何原本属于 Python 的应用层逻辑，优先迁移到 `context`、`utils` 或独立 crate。
