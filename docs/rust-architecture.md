# Rust 架构文档

## 定位

Rust 是桌面应用主后端与核心业务层。

## 负责内容

- Tauri 应用启动与管理
- 配置读取与持久化
- 渠道目录与通道状态
- sidecar 生命周期
- 渠道连接、断开、重启策略
- AI Agent 工具链（`rig-core` 编排 + 内置工具）
- 与前端的 IPC 契约
- 持久化业务状态

## 分层

- `src/cmd`
  - Tauri 命令入口（薄）
- `src/context`
  - 共享状态、sidecar 协调、**业务编排**（渠道、Agent 运行时、收件箱等）
- `src/services`
  - 可复用业务能力（Agent、Bridge、渠道服务）
- `src/events`
  - 事件契约与发射
- `src/utils`
  - **桌面应用侧**无 Store 工具（路径、日志、渠道解析等）
- `src/config`、`src/io`、`src/channel_runtime`、`src/process_runtime`
  - 配置模型、带日志文件 IO、渠道消息规则、子进程/RPC 基础设施
- `src/python`
  - 与 `channel_agent/` 的**唯一**互操作层（sidecar + markitdown）
- `src/cli`
  - `sf` 无头命令（与桌面共享 `services::*`）

## 单 crate 结构（重要）

`src-tauri` 现为**单一 workspace 成员**（`Cargo.toml` 中 `members = ["."]`），**不再维护** `src-tauri/crates/*` 子 crate。

| 模块                | 定位                                                             | 说明                                                |
| ------------------- | ---------------------------------------------------------------- | --------------------------------------------------- |
| `services/agent/`   | Agent 工具链、知识库、记忆、技能、`rig` LLM 编排                 | 原 `crates/agent`，由 `context::agent_runtime` 编排 |
| `services/bridge/`  | Bot 路由、`AgentBridge`、配置同步                                | 原 `crates/bridge`                                  |
| `services/channel/` | 渠道领域服务（配置读写等）                                       | 与 `context/channel` 配合                           |
| `config/`           | `config.json` 模型、Provider 目录、Context/Reply 契约            | 原 `crates/models` 配置与契约部分                   |
| `io/`               | 带审计日志的 `std::fs` 封装（`lib.rs` 中 `pub use io as fs_io`） | 原 `crates/fs_io`                                   |
| `channel_runtime/`  | 渠道消息前缀/关键词/回复装饰（纯算法）                           | 原 `crates/channel_runtime`                         |
| `process_runtime/`  | 子进程 spec、一次性命令、stdio NDJSON RPC 运行时                 | 原 `crates/process_runtime`                         |
| `cli/`              | `sf` 命令实现                                                    | 原 `crates/cli`                                     |
| `python/`           | sidecar 与 markitdown 调用                                       | 见 `rust-folder-structure.md`                       |

**禁止：** 在多个模块内复制同一份工具实现（例如多份 `fs.rs`）。文件 IO 统一 `crate::io`（或 `crate::fs_io`）。

**目标结构（当前形态）：**

```
src-tauri/
  src/
    cmd/              # Tauri command 薄入口（desktop）
    context/          # 共享状态与运行时编排（desktop）
    services/         # agent / bridge / channel
    config/           # 配置与 Provider 契约
    io/               # 带日志文件 IO
    channel_runtime/  # 渠道消息纯算法
    process_runtime/  # 子进程与 stdio RPC 基础设施
    python/           # Python 互操作（sidecar + markitdown）
    cli/              # sf 命令
    utils/            # 无 Store 工具
    events/           # 事件（desktop）
  binaries/           # PyInstaller channel-sidecar 产物
```

CLI（`sf`）通过 `default-features = false` 依赖 `services::*`、`config`、`io` 等，不链接 Tauri desktop feature。

## 与 Python 的边界

1. Rust 决定策略。
2. Python 只执行 SDK 适配。
3. 前端优先面对 Rust，而不是面对 Python。
4. **集成方式固定为进程隔离**：Tauri sidecar（长驻）+ 脚本子进程（markitdown）；**不使用 PyO3 嵌入解释器**。

## LLM 编排

- Agent 对话与工具循环由 `services/agent/rig/`（`rig-core`）承载。
- Provider 凭据与 `config.json` 读写走 `config::provider_catalog`。
- 桌面运行时由 `context::agent_runtime` + `services::bridge` 组装。
