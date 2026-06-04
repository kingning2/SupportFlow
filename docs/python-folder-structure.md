# Python 文件夹结构

## 目标

Python 目录只保留渠道 SDK 适配与 `markitdown` 最小骨架，不再承载应用编排层。

## 允许保留的核心目录

- `src-tauri/channel_agent/channel/wechat/`
  - 个人微信 SDK 适配代码
- `src-tauri/channel_agent/channel/wework/`
  - 个人企业微信 SDK 适配代码
- `src-tauri/channel_agent/lib/itchat/`
  - vendored `itchat`
- `src-tauri/channel_agent/scripts/markitdown_convert.py`
  - 文档转 Markdown 脚本

## 允许保留的核心骨架文件

- `src-tauri/channel_agent/channel/rust_ipc.py`
- `src-tauri/channel_agent/channel/rpc_handlers.py`
- `src-tauri/channel_agent/channel/stdio_server.py`
- `src-tauri/channel_agent/config.py`
- `src-tauri/channel_agent/requirements-markitdown.txt`
- `src-tauri/channel_agent/requirements-sidecar.txt`
- `src-tauri/channel_agent/requirements-wework.txt`

## 结构原则

1. `channel/` 下只放 sidecar 启动、Rust RPC 对接、以及渠道适配代码。
2. `wechat/`、`wework/` 目录只处理各自 SDK 登录、消息解析、消息发送、媒体下载。
3. 共有工具尽量极少；如果不是 SDK 适配必需，优先迁移到 Rust。
4. 不再新增测试目录、复现脚本、旧控制台路由、旧渠道工厂层、旧 AI/模型层。
