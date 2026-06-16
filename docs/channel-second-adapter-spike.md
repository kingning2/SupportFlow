# 第二渠道接入 Spike（Mock）

T008 验证：除企微外可注册并走通消息链路。

## 已验证路径

| 步骤          | Mock                                    | WeWork            |
| ------------- | --------------------------------------- | ----------------- |
| Rust 注册     | `registry.rs` → `mock`                  | `wework`          |
| Python 适配器 | `channel/mock/mock_channel.py`          | `channel/wework/` |
| connect       | `channel_python_channels_post`          | 同左              |
| 入站消息      | 启动后自动注入                          | SDK 回调          |
| Inbox 持久化  | `channel.message` → `ChannelInboxStore` | 同左              |
| Agent 回复    | `agent.reply` RPC                       | 同左              |
| 出站 send     | 日志输出                                | ntwork SDK        |

## 新增渠道 Checklist

1. **Python**：实现 `ChannelAdapter` / `ChatChannel`；`register_channel(id, factory)`；可选 `register_extension_rpc`
2. **Rust**：在 `services/channel/registry.rs` 追加 `ChannelDef`（fields、restart_keys、capabilities）
3. **配置**：`channel_specific.{id}` 块随 connect/save 自动写入
4. **前端**（可选）：`CHANNEL_IDS`、catalog 卡片
5. **测试**：registry 单测 + 手动 connect mock 验证 inbox

## 限制（见 T012）

- 当前 **单 sidecar 进程**、**单活跃渠道线程**；`channel_type=wework,mock` 可共存于配置，但同时运行依赖 sidecar 多 slot（未实现）。
- Mock 不调用真实外部 API，适合 CI / 开发验收。

## 工作量估算

| 真实渠道（如 Telegram Bot） | 人日    | 风险            |
| --------------------------- | ------- | --------------- |
| Python SDK 适配             | 2–3     | SDK 稳定性      |
| Rust 注册 + 配置字段        | 0.5     | 低              |
| 账号/登录 UI                | 1–2     | OAuth / token   |
| Inbox + Agent 联调          | 1       | 已有通路        |
| **合计**                    | **~5d** | 视 SDK 文档质量 |

Mock spike 本身约 **0.5d**（本任务）。
