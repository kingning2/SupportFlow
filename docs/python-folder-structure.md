# Python 文件夹结构

## 目标

Python 目录只保留渠道 SDK 适配与 `markitdown` 最小骨架，不再承载应用编排层。

**位置：仓库根目录 `channel_agent/`**（不在 `src-tauri/` 内）。

Rust 通过 `src-tauri/src/python/` 调用 Python（sidecar RPC、MarkItDown 子进程），不在 `context` 里散落路径与进程代码。

## 允许保留的核心目录

- `channel_agent/channel/wework/`
  - 企业微信 SDK 适配（`ntwork`）
- `channel_agent/scripts/markitdown_convert.py`
  - 文档转 Markdown 脚本（**非** sidecar 入口）

## 允许保留的核心骨架文件

- `channel_agent/channel/rust_ipc.py` — 双向 NDJSON RPC
- `channel_agent/channel/rpc_handlers.py` — 最小 RPC 分发
- `channel_agent/channel/stdio_server.py` — sidecar 启动入口
- `channel_agent/channel/channel_manager.py` — sidecar 内渠道实例化（非桌面策略）
- `channel_agent/config.py`
- `channel_agent/requirements-markitdown.txt`
- `channel_agent/requirements-sidecar.txt`
- `channel_agent/requirements-wework.txt`

## Rust 调用入口（`src-tauri/src/python/`）

| 模块                     | 职责                                                                           |
| ------------------------ | ------------------------------------------------------------------------------ |
| `paths.rs`               | 解析 `channel_agent/`、sidecar 二进制、MarkItDown 脚本路径                     |
| `paths_desktop.rs`       | 依赖 Tauri 资源的 MarkItDown 路径（desktop）                                   |
| `markitdown.rs`          | 单次子进程调用 `markitdown_convert.py`                                         |
| `sidecar/mod.rs`         | `ChannelPythonSidecar`、领域 RPC（`channel_start`、`wework_sync_contacts` 等） |
| `sidecar/spawn.rs`       | 启动模式解析、`spawn_sidecar`                                                  |
| `sidecar/handler.rs`     | Python 入站 RPC / 事件                                                         |
| `sidecar/tauri_shell.rs` | Tauri `externalBin` 的 stdin/stdout 适配                                       |

底层进程读写与 RPC 超时在 `src/process_runtime/`；Python 层不实现第二套 RPC 框架。

## Sidecar 构建与产物

```bash
pnpm run setup:channel-sidecar-dev   # 仅安装 sidecar Python 依赖
pnpm run build:channel-sidecar       # PyInstaller → src-tauri/binaries/
```

开发态可不构建 exe，Rust 自动回退 `python -m channel`（工作目录为 `channel_agent/`）。

## 结构原则

1. `channel/` 下只放 sidecar 启动、Rust RPC 对接、以及渠道适配代码。
2. `wework/` 处理 SDK 的登录、消息解析、发送、媒体下载。
3. `markitdown_convert.py` 只服务 Rust 单次子进程调用，不并入 sidecar 主循环。
4. 共有工具尽量极少；如果不是 SDK 适配必需，优先迁移到 Rust。
5. 不再新增测试目录、复现脚本、旧控制台路由、旧渠道工厂层、旧 AI/模型层。
6. **不使用 PyO3**；Python 代码始终作为独立进程运行。
