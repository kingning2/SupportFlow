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

- `src/cmd`
  - Tauri 命令入口（薄）
- `src/context`
  - 共享状态、sidecar 协调、**业务编排**（渠道、Agent 运行时、收件箱等）
- `src/events`
  - 事件契约与发射
- `src/utils`
  - **桌面应用侧**无 Store 工具（路径、日志、渠道解析等）
- `crates/*`
  - **仅基础设施与协议适配**，不放桌面业务编排（见下节）

## `crates/` 与 `src/` 边界（重要）

| 放在 `crates/`                    | 放在 `src/`（`context` / `utils`）             |
| --------------------------------- | ---------------------------------------------- |
| 无 Tauri、无 Store 的纯函数库     | 依赖 `AppHandle`、`.manage` Store 的逻辑       |
| 可独立单测的协议/算法             | 渠道连接、sidecar 生命周期、收件箱持久化       |
| 被 CLI 与桌面端共同依赖的 IO/路径 | `agent_runtime`、wework 账号、license 等应用态 |

**当前 `crates/` 成员定位：**

| Crate             | 定位                                               | 业务迁移方向                                    |
| ----------------- | -------------------------------------------------- | ----------------------------------------------- |
| `fs_io`           | 带审计日志的 `std::fs` 封装                        | 保持                                            |
| `channel_runtime` | 渠道消息前缀/关键词/回复装饰（纯算法）             | 保持                                            |
| `models`          | LLM Provider HTTP、session、`config.json` 字段模型 | 逐步只保留「协议 + 配置读写」，桌面策略回 `src` |
| ~~`agent`~~       | 已迁入 `src/services/agent/`                       | 由 `context::agent_runtime` 编排                |
| ~~`bridge`~~      | 已迁入 `src/services/bridge/`                      | 由 `context::agent_runtime` / CLI 共用          |
| `cli`             | `sf` 二进制入口                                    | 保持；只调 `src` 暴露的 API 或基础设施 crate    |

**禁止：** 在多个 crate 内复制同一份工具实现（例如曾经的 5 份 `fs.rs`）。统一使用 `fs_io`。

**目标结构（演进中）：**

```
src-tauri/
  src/           ← 全部桌面业务与编排
  crates/
    fs_io/       ← 基础设施
    channel_runtime/
    models/      ← 协议适配（瘦身）
    cli/         ← 二进制
    (无 agent/bridge — 已在 src/services/)
```

## 与 Python 的边界

1. Rust 决定策略。
2. Python 只执行 SDK 适配。
3. 前端优先面对 Rust，而不是面对 Python。
