# Python 架构文档

## 定位

Python sidecar 是渠道 SDK 适配进程，不是应用主后端。

## 负责内容

- `wechat` 登录、扫码、消息回调、发送
- `wework` 登录、消息回调、发送、联系人同步触发
- 媒体下载与少量 SDK 必需格式转换
- `markitdown` 文档转 Markdown
- Rust 与 sidecar 之间的最小 RPC

## 不负责内容

- 前端频道目录
- 前端 API 路由
- 渠道配置策略
- 渠道重启策略
- AI Agent 编排
- 模型配置与调用
- 持久化业务状态

## 目标形态

Python 最终只剩：

- `wechat_channel + wechat_message + itchat vendored`
- `wework_channel + wework_message + run + ntwork glue`
- `rust_ipc / rpc_handlers / stdio_server / config`
- `markitdown_convert.py`

## 迁移原则

如果某段 Python 代码不是 SDK 适配必需，就应该迁移到 Rust 或删除。
