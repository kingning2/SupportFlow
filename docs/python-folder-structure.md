# Python 文件夹结构

## 目标

Python 目录只保留渠道 SDK 适配与 `markitdown` 最小骨架，不再承载应用编排层。

**位置：仓库根目录 `channel_agent/`**（不在 `src-tauri/` 内）。

Rust 通过 `src-tauri/src/python/` 调用 Python（sidecar RPC、MarkItDown），不在 `context` 里散落路径与进程代码。

## 允许保留的核心目录

- `channel_agent/channel/wechat/`
  - 个人微信 SDK 适配代码
- `channel_agent/channel/wework/`
  - 企业微信 SDK 适配代码
- `channel_agent/lib/itchat/`
  - vendored `itchat`
- `channel_agent/scripts/markitdown_convert.py`
  - 文档转 Markdown 脚本

## 允许保留的核心骨架文件

- `channel_agent/channel/rust_ipc.py`
- `channel_agent/channel/rpc_handlers.py`
- `channel_agent/channel/stdio_server.py`
- `channel_agent/config.py`
- `channel_agent/requirements-markitdown.txt`
- `channel_agent/requirements-sidecar.txt`
- `channel_agent/requirements-wework.txt`

## Rust 调用入口（`src-tauri/src/python/`）

| 模块               | 职责                                                       |
| ------------------ | ---------------------------------------------------------- |
| `paths.rs`         | 解析 `channel_agent/`、sidecar 二进制、MarkItDown 脚本路径 |
| `markitdown.rs`    | 调用 `markitdown_convert.py`                               |
| `sidecar.rs`       | 启动 sidecar、stdio NDJSON RPC                             |
| `client.rs`        | `channel.start` / `wework.sync_contacts` 等便捷封装        |
| `paths_desktop.rs` | 依赖 Tauri 资源的 MarkItDown 路径                          |

## 结构原则

1. `channel/` 下只放 sidecar 启动、Rust RPC 对接、以及渠道适配代码。
2. `wechat/`、`wework/` 目录只处理各自 SDK 登录、消息解析、消息发送、媒体下载。
3. 共有工具尽量极少；如果不是 SDK 适配必需，优先迁移到 Rust。
4. 不再新增测试目录、复现脚本、旧控制台路由、旧渠道工厂层、旧 AI/模型层。
