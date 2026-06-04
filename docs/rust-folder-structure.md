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

- `src-tauri/crates/agent/`
  - AI Agent 工具链
- `src-tauri/crates/bridge/`
  - Rust 业务桥接能力
- `src-tauri/crates/channel_runtime/`
  - 渠道通用消息处理逻辑
- `src-tauri/crates/models/`
  - 配置模型与提供商模型

## 结构原则

1. `cmd` 只做命令入口与参数/返回值定义。
2. `context` 只做共享状态、sidecar 运行时协调、状态同步。
3. `utils` 只做无状态工具逻辑。
4. 任何原本属于 Python 的应用层逻辑，优先迁移到 `context`、`utils` 或独立 crate。
