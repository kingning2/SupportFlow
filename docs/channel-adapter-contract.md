# Channel Adapter Contract

统一桌面渠道适配器契约：Rust 编排 + Python SDK 薄适配 + stdio NDJSON IPC。

## 能力矩阵

| 能力                   | 负责层            | Rust                                       | Python                  | IPC / 事件                                             |
| ---------------------- | ----------------- | ------------------------------------------ | ----------------------- | ------------------------------------------------------ |
| **connect**            | Rust 编排         | `services/channel/config::connect_channel` | —                       | Tauri `channel_python_channels_post` → `channel.start` |
| **disconnect**         | Rust 编排         | `disconnect_channel`                       | —                       | `channel.stop`                                         |
| **list_conversations** | Rust              | `ChannelInboxStore::snapshot`              | —                       | Tauri `channel_inbox_snapshot`                         |
| **send**               | Python SDK        | Agent 回复经 `ChatChannel._send`           | `ChannelAdapter.send`   | —                                                      |
| **on_message**         | Python SDK → Rust | `channel.process` / `agent.reply`          | `ChatChannel.produce`   | `channel.message` (notify)                             |
| **health**             | 双方              | `ChannelBridge::is_active_phase`           | `ChannelAdapter.health` | `channel.notify` phase=`ready`                         |

## 生命周期 Phase

Sidecar 通过 `channel.notify` 推送，Rust 写入 `ChannelStatusStore` 并 emit `channel/status-changed`。

| Phase                            | 含义         | 前端 login_status            |
| -------------------------------- | ------------ | ---------------------------- |
| `starting`                       | SDK 初始化中 | `waiting_scan`               |
| `waiting_login` / `waiting_scan` | 等待登录     | `waiting_scan`               |
| `scanned`                        | 已扫码       | `scanned`                    |
| `logged_in`                      | 已登录       | `logged_in`                  |
| `syncing`                        | 后台同步     | `logged_in`                  |
| `ready`                          | 可收发消息   | `logged_in`（`active=true`） |
| `error`                          | 启动失败     | `unknown`                    |
| `stopped`                        | 已停止       | `unknown`                    |

常量定义：`src-tauri/src/services/channel/contract.rs` → `phase::*`

## 错误码

| Code                          | 场景                                |
| ----------------------------- | ----------------------------------- |
| `channel.unknown`             | 未注册的 `channel_type`             |
| `channel.unknown_action`      | connect/save/disconnect 以外 action |
| `channel.sidecar_not_running` | sidecar 未启动                      |
| `channel.config_invalid`      | 配置字段无效                        |
| `channel.startup_failed`      | SDK 启动失败（phase=`error`）       |
| `channel.send_failed`         | 出站发送失败                        |

## IPC 方法对照

### Rust → Python（sidecar outbound）

| Method                 | Params        | Result                                                    |
| ---------------------- | ------------- | --------------------------------------------------------- |
| `channel.start`        | `{ channel }` | `{ status, channel }`                                     |
| `channel.stop`         | `{ channel }` | `{ status, channel }`                                     |
| `channel.restart`      | `{ channel }` | `{ status, channel }`                                     |
| `wework.sync_contacts` | `{}`          | `{ status, started }`（扩展 RPC，注册于 Python registry） |
| `ping`                 | `{}`          | `{ status, pong }`                                        |

### Python → Rust（sidecar inbound）

| Method                  | Params                  | Result                       |
| ----------------------- | ----------------------- | ---------------------------- |
| `agent.reply`           | query + context         | `{ content, reply_type, … }` |
| `channel.process`       | `{ context, config }`   | `ChannelRuntimeResult` JSON  |
| `channel.decorate_text` | `{ text, meta }`        | `{ text }`                   |
| `channel.extract_media` | `{ text, limit }`       | `{ items: [{ url, kind }] }` |
| `channel.notify`        | `{ channel, phase, … }` | `{ status }`                 |
| `channel.message`       | inbox payload           | `{ status }`                 |

### Tauri 前端

| Command                        | 说明                                           |
| ------------------------------ | ---------------------------------------------- |
| `agent_get_channel_catalog`    | 读取 `registry::all_channel_defs()` 构建的目录 |
| `channel_python_channels_post` | connect / disconnect / save                    |
| `channel_inbox_snapshot`       | 会话列表 + 消息                                |

## 类型对照

| 概念     | Rust                               | Python                  | TypeScript                     |
| -------- | ---------------------------------- | ----------------------- | ------------------------------ |
| 渠道 id  | `ChannelTypeId`                    | `channel_type: str`     | `ChannelTypeId`（typeshare）   |
| 适配器   | `ChannelDef` + registry            | `ChannelAdapter` ABC    | `ChannelCatalogEntry`          |
| 消息规则 | `channel_runtime::process_message` | `_rust_process_context` | —                              |
| 状态事件 | `ChannelStatusChangedPayload`      | `notify_channel_status` | event `channel/status-changed` |

## 注册扩展新渠道

新增第二渠道时**无需修改** `ProcessHub` 核心逻辑。

### Rust

1. 在 `services/channel/registry.rs` 的 `CHANNEL_DEFS` 追加 `ChannelDef`（fields、restart_keys、capabilities）。
2. 若需 feature gate，使用 `#[cfg(feature = "channel-xxx")]`。
3. 渠道专属 store（如企微账号库）放在 `context/channel/`，catalog 中按 name 分支 enrich。

### Python

1. 实现 `ChannelAdapter`（通常继承 `ChatChannel`）。
2. 在 `channel/<name>/__init__.py` 调用 `register_channel("<name>", factory, cleanup=…)`。
3. 可选：`register_extension_rpc("xxx.action", handler)`。
4. 在 `channel/__init__.py` 或包入口 import 该子包以触发注册。

### 前端

1. 运行 `pnpm run contracts` 同步 `ChannelTypeId`。
2. 更新 `packages/shared/src/tauri-bridge/enums/dev-channel.ts` 的 `CHANNEL_IDS`（T007 将自动化）。

## 相关文件

- `src-tauri/src/services/channel/registry.rs` — 注册表与 config schema
- `src-tauri/src/services/channel/contract.rs` — phase / error / IPC 常量
- `src-tauri/src/context/channel/bridge.rs` — 活跃渠道与 phase 映射
- `src-tauri/src/channel_runtime/` — 消息触发与回复装饰（纯函数）
- `channel_agent/channel/adapter.py` — Python ABC
- `channel_agent/channel/registry.py` — Python 工厂注册
