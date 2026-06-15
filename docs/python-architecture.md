# Python 架构文档

## 定位

Python sidecar 是渠道 SDK 适配进程，不是应用主后端。

## 与 Rust 的集成方式（固定）

| 场景            | 方式               | 说明                                                                                         |
| --------------- | ------------------ | -------------------------------------------------------------------------------------------- |
| 企业微信渠道    | **Tauri sidecar**  | PyInstaller 打包的 `channel-sidecar` 或开发态 `python -m channel`；双向 **stdio NDJSON RPC** |
| 文档转 Markdown | **单次脚本子进程** | Rust 调用 `markitdown_convert.py`，用完即退                                                  |
| 嵌入解释器      | **不使用 PyO3**    | 保持进程隔离：SDK 崩溃不拖垮桌面应用，`ntwork` 线程模型与 Rust async 解耦                    |

长驻 sidecar 与 markitdown 是**两套进程模型**，不得合并为同一 Python 入口。

## 负责内容

- `wework` 登录、消息回调、发送、联系人同步触发
- 媒体下载与少量 SDK 必需格式转换
- `markitdown` 文档转 Markdown（由 Rust 以子进程方式调用，不在 sidecar 内编排）
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

- `wework_channel + wework_message + run + ntwork glue`
- `rust_ipc / rpc_handlers / stdio_server / config`
- `markitdown_convert.py`

## 迁移原则

如果某段 Python 代码不是 SDK 适配必需，就应该迁移到 Rust 或删除。

## Sidecar 启动链（概要）

1. Rust `context::AgentRuntime` 延迟启动 sidecar。
2. `python::sidecar::spawn_sidecar` 解析模式：自定义 exe → 开发源码 → Tauri `externalBin`。
3. Python `stdio_server.run_stdio_server` 加载配置、启动 `ChannelManager`、进入 NDJSON 循环。
4. Python 通过 `rust_ipc.notify_rust` / `call_rust` 回调 Rust；Rust 通过 RPC 调用 `channel.start` 等。
