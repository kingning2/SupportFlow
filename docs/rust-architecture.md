# Rust 架构文档

## 定位

Rust 是桌面应用主后端与核心业务层。

## 负责内容

- Tauri 应用启动与管理
- 配置读取与持久化
- 渠道目录与通道状态
- sidecar 生命周期
- 渠道连接、断开、重启策略
- AI Agent 工具链
- 与前端的 IPC 契约
- 持久化业务状态

## 分层

- `cmd`
  - Tauri 命令入口
- `context`
  - 共享状态、sidecar 协调、运行态编排
- `events`
  - 事件契约
- `utils`
  - 可复用工具逻辑
- `crates/*`
  - 更独立的业务能力

## 与 Python 的边界

1. Rust 决定策略。
2. Python 只执行 SDK 适配。
3. 前端优先面对 Rust，而不是面对 Python。
