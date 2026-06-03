# Rust API 参考（Command / Event / 内部方法）

> 自动生成基准：`src-tauri/src/lib.rs` 的 `generate_handler!` 与 `packages/shared/src/tauri-bridge/`。  
> 新增 Command 后请同步更新本文档（或运行 `pnpm run generate:contracts` 后对照 diff）。

前端调用约定：

- **Command**：`invokeWrapper(TauriCmd.Xxx, …)`，封装函数在 `@supportflow/shared/tauri-bridge/cmd/*`
- **Event**：`TauriEvent` + `tauriOn` / `TauriEventProvider`
- **类型**：共享 DTO 来自 `@supportflow/shared/contracts`（由 Rust `#[typeshare]` 生成）

---

## 快速索引：全部 Command（43 个）

| 模块  | Rust 命令名                       | `TauriCmd`                    | 前端封装                                      |
| ----- | --------------------------------- | ----------------------------- | --------------------------------------------- |
| 语言  | `get_lang`                        | `GetLang`                     | `getLang()`                                   |
| 语言  | `set_lang`                        | `SetLang`                     | `setLang(lang)`                               |
| 语言  | `get_language_resource_bundle`    | `GetLanguageResourceBundle`   | `getLanguageResourceBundle(lang)`             |
| 会话  | `get_app_session`                 | `GetAppSession`               | `getAppSession()`                             |
| 日志  | `log_fe`                          | `LogFe`                       | （legacy invoke，推荐 Event `FeLog`）         |
| 日志  | `log_fe_req`                      | `LogFeReq`                    | （legacy invoke，推荐 Event `FeLogReq`）      |
| 窗口  | `open_modal_window`               | `OpenModalWindow`             | `openModalWindow(args)`                       |
| 窗口  | `close_modal_window`              | `CloseModalWindow`            | `closeModalWindow(label)`                     |
| 窗口  | `modal_window_ready`              | `ModalWindowReady`            | `notifyModalWindowReady(label)`               |
| 窗口  | `preload_modal_window`            | `PreloadModalWindow`          | `preloadModalWindow()`                        |
| 授权  | `license_get_status`              | `LicenseGetStatus`            | `getLicenseStatus()`                          |
| 授权  | `license_apply_activation`        | `LicenseApplyActivation`      | `applyLicenseActivation(token)`               |
| Agent | `agent_get_console_state`         | `AgentGetConsoleState`        | `getAgentConsoleState()`                      |
| Agent | `agent_send_message`              | `AgentSendMessage`            | `sendAgentMessage(body)`                      |
| Agent | `agent_cancel`                    | `AgentCancel`                 | `cancelAgentMessage(requestId)`               |
| Agent | `agent_clear_context`             | `AgentClearContext`           | `clearAgentContext()`                         |
| Agent | `agent_new_session`               | `AgentNewSession`             | `newAgentSession()`                           |
| Agent | `agent_refresh_skills`            | `AgentRefreshSkills`          | `refreshAgentSkills()`                        |
| Agent | `agent_update_provider`           | `AgentUpdateProvider`         | `updateAgentProvider(body)`                   |
| Agent | `agent_clear_provider`            | `AgentClearProvider`          | `clearAgentProvider(body)`                    |
| Agent | `agent_set_chat_model`            | `AgentSetChatModel`           | `setAgentChatModel(body)`                     |
| Agent | `agent_list_sessions`             | `AgentListSessions`           | `listAgentSessions()`                         |
| Agent | `agent_list_memory`               | `AgentListMemory`             | `listAgentMemory()`                           |
| Agent | `agent_read_memory`               | `AgentReadMemory`             | `readAgentMemory(filename)`                   |
| Agent | `agent_list_knowledge`            | `AgentListKnowledge`          | `listAgentKnowledge()`                        |
| Agent | `agent_read_knowledge`            | `AgentReadKnowledge`          | `readAgentKnowledge(path)`                    |
| Agent | `agent_get_knowledge_graph`       | `AgentGetKnowledgeGraph`      | `getAgentKnowledgeGraph()`                    |
| Agent | `agent_upload_knowledge`          | `AgentUploadKnowledge`        | `uploadAgentKnowledge(files, category?)`      |
| Agent | `agent_pick_and_upload_knowledge` | `AgentPickAndUploadKnowledge` | `pickAndUploadKnowledge(category?)`           |
| Agent | `agent_list_channels`             | `AgentListChannels`           | `listAgentChannels()`                         |
| Agent | `agent_get_channel_catalog`       | `AgentGetChannelCatalog`      | `fetchChannels()`（推荐）                     |
| Agent | `agent_channel_action`            | `AgentChannelAction`          | `channelAction(body)`（推荐）                 |
| Agent | `agent_channel_console_api`       | `AgentChannelConsoleApi`      | `fetchChannelConsoleApi(path, method, body?)` |
| Agent | `agent_list_tasks`                | `AgentListTasks`              | `listAgentTasks()`                            |
| Agent | `agent_get_logs_status`           | `AgentGetLogsStatus`          | `getAgentLogsStatus()`                        |
| Agent | `agent_read_logs`                 | `AgentReadLogs`               | `readAgentLogs({ limit? })`                   |
| Agent | `agent_start_log_stream`          | `AgentStartLogStream`         | `startAgentLogStream()`                       |
| Agent | `agent_stop_log_stream`           | `AgentStopLogStream`          | `stopAgentLogStream()`                        |
| 企微  | `wework_list_accounts`            | `WeworkListAccounts`          | `weworkListAccounts()`                        |
| 企微  | `wework_upsert_account`           | `WeworkUpsertAccount`         | `weworkUpsertAccount(account)`                |
| 企微  | `wework_delete_account`           | `WeworkDeleteAccount`         | `weworkDeleteAccount(id)`                     |
| 企微  | `wework_get_active_account_id`    | `WeworkGetActiveAccountId`    | `weworkGetActiveAccountId()`                  |
| 企微  | `wework_set_active_account_id`    | `WeworkSetActiveAccountId`    | `weworkSetActiveAccountId(id)`                |

**授权门禁**：除「语言 / 会话 / 窗口 / 授权 / 日志」外，Agent 与企微相关 Command 均会调用 `LicenseStore::require_valid()`，未激活时返回错误。

---

## 前端调用示例

```ts
import { TauriCmd } from "@supportflow/shared/tauri-bridge/enums";
import { invokeWrapper } from "@supportflow/shared/tauri-bridge/cmd/invoke";
import {
  getAgentConsoleState,
  sendAgentMessage,
  pickAndUploadKnowledge
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { getAppSession } from "@supportflow/shared/tauri-bridge/cmd/session";

// 推荐：用封装函数
const state = await getAgentConsoleState();
const { requestId, sessionId } = await sendAgentMessage({ message: "你好" });

// 也可直接用 invokeWrapper
await invokeWrapper(TauriCmd.SetLang, { lang: "cn" });
```

封装文件路径：

| 文件                             | 导出                                                                                  |
| -------------------------------- | ------------------------------------------------------------------------------------- |
| `cmd/lang.ts`                    | `getLang`, `setLang`, `getLanguageResourceBundle`                                     |
| `cmd/session.ts`                 | `getAppSession`                                                                       |
| `cmd/window.ts`                  | `openModalWindow`, `closeModalWindow`, `notifyModalWindowReady`, `preloadModalWindow` |
| `cmd/license.ts`                 | `getLicenseStatus`, `applyLicenseActivation`                                          |
| `cmd/agent.ts`                   | Agent 控制台全部 API                                                                  |
| `cmd/channel-python-channels.ts` | `fetchChannels`, `channelAction`, `fetchChannelConsoleApi`                            |
| `cmd/wework-accounts.ts`         | 企微账号 CRUD                                                                         |

---

## Command 详情

### 语言与会话

#### `get_lang`

- **说明**：读取当前语言代码（`"cn"` | `"en"`）。
- **参数**：无
- **返回**：`string`
- **Rust**：`cmd/lang.rs` → `context/session`

#### `set_lang`

- **说明**：切换语言并广播 `session/changed`。
- **参数**：`lang: string`
- **返回**：`void`
- **Rust**：`cmd/lang.rs` → `context/session::set_current_language`

#### `get_language_resource_bundle`

- **说明**：加载 `resources/languages/{code}.json` i18n 包。
- **参数**：`language: string`
- **返回**：`serde_json::Value`（嵌套 JSON 对象）

#### `get_app_session`

- **说明**：跨 Webview 会话快照（含当前语言等）。
- **参数**：无
- **返回**：`AppSession`（typeshare → `contracts`）
- **配套 Event**：`session/changed`

---

### 窗口（Modal）

#### `open_modal_window`

- **参数**：`path`, `title?`, `width?`, `height?`, `label?`
- **返回**：`string`（窗口 label）
- **Rust**：`utils/window`

#### `close_modal_window`

- **参数**：`label: string`
- **返回**：`void`

#### `modal_window_ready`

- **说明**：Modal Webview 加载完成后通知 Rust 显示窗口。
- **参数**：`label: string`

#### `preload_modal_window`

- **说明**：主窗空闲时后台预热隐藏 Modal Webview。

---

### 授权（License）

#### `license_get_status`

- **说明**：启动时计算的机器码 + 授权有效性快照。
- **返回**：`LicenseStatusDto` `{ machineCode, valid, reason? }`

#### `license_apply_activation`

- **说明**：校验激活码、持久化、刷新授权态。
- **参数**：`token: string`
- **返回**：`LicenseStatusDto`

---

### Agent 控制台

流式回复通过 **Event** 推送，不阻塞 Command 返回：

1. `sendAgentMessage` → 立即返回 `{ requestId, sessionId }`
2. 监听 `agent/stream-chunk` 接收增量
3. 监听 `agent/run-finished` 接收结束态

#### 对话与 Session

| Command                   | 参数                      | 返回                       | 说明                                                      |
| ------------------------- | ------------------------- | -------------------------- | --------------------------------------------------------- |
| `agent_get_console_state` | —                         | `AgentConsoleState`        | 控制台 bootstrap 聚合态（providers、skills、messages 等） |
| `agent_send_message`      | `{ message, sessionId? }` | `{ requestId, sessionId }` | 提交用户消息，后台流式执行                                |
| `agent_cancel`            | `{ requestId }`           | `void`                     | 按 requestId 取消运行中任务                               |
| `agent_clear_context`     | —                         | `void`                     | 清空当前 session 内存上下文                               |
| `agent_new_session`       | —                         | `string`                   | 创建并切换到新 sessionId                                  |
| `agent_list_sessions`     | —                         | `AgentSessionSummary[]`    | 持久化 session 索引                                       |

#### Provider / Model / Skills

| Command                 | 参数                                            | 返回          | 说明                         |
| ----------------------- | ----------------------------------------------- | ------------- | ---------------------------- |
| `agent_update_provider` | `{ providerId, apiKey?, apiBase?, apiBaseSet }` | `void`        | 更新 provider 凭据           |
| `agent_clear_provider`  | `{ providerId }`                                | `void`        | 清除 provider 凭据           |
| `agent_set_chat_model`  | `{ providerId?, model? }`                       | `void`        | 设置当前 chat provider/model |
| `agent_refresh_skills`  | —                                               | `SkillItem[]` | 刷新 skill 注册表            |

#### Memory（工作区 memory/）

| Command             | 参数           | 返回                                        |
| ------------------- | -------------- | ------------------------------------------- |
| `agent_list_memory` | —              | `{ filename, itemType, size, updatedAt }[]` |
| `agent_read_memory` | `{ filename }` | `{ filename, content }`                     |

#### Knowledge（工作区 knowledge/）

| Command                           | 参数                                         | 返回                         | 说明                                      |
| --------------------------------- | -------------------------------------------- | ---------------------------- | ----------------------------------------- |
| `agent_list_knowledge`            | —                                            | `{ path, title }[]`          | 列出知识库文档                            |
| `agent_read_knowledge`            | `{ path }`                                   | `{ path, content }`          | 按相对路径读取                            |
| `agent_get_knowledge_graph`       | —                                            | `{ nodes, links }`           | Markdown 交叉引用图谱                     |
| `agent_upload_knowledge`          | `{ files: [{ filename, data }], category? }` | `AgentKnowledgeUploadResult` | 前端传字节，Rust 解析→Markdown→索引       |
| `agent_pick_and_upload_knowledge` | `{ category? }`                              | `AgentKnowledgeUploadResult` | **原生文件选择器**，Rust 直读磁盘（推荐） |

`AgentKnowledgeUploadResult` 字段：`results[]`, `errors[]`, `count`, `memorySynced`。

#### Channels（渠道）

| Command                     | 参数                           | 返回                        | 说明                                                        |
| --------------------------- | ------------------------------ | --------------------------- | ----------------------------------------------------------- |
| `agent_list_channels`       | —                              | `{ name, active, label }[]` | 简单列表（legacy）                                          |
| `agent_get_channel_catalog` | —                              | `JSON`                      | 代理 Python `GET /api/channels`                             |
| `agent_channel_action`      | `{ action, channel, config? }` | `JSON`                      | 代理 Python `POST /api/channels`（connect/disconnect/save） |
| `agent_channel_console_api` | `{ path, method, body? }`      | `JSON`                      | QR 登录、飞书注册等 console API                             |

前端推荐用 `channel-python-channels.ts` 的 `fetchChannels()` / `channelAction()`，内含 status 校验。

#### Tasks / Logs

| Command                  | 参数         | 返回                                  | 说明                                       |
| ------------------------ | ------------ | ------------------------------------- | ------------------------------------------ |
| `agent_list_tasks`       | —            | `{ id, name, enabled, nextRunAt? }[]` | 定时任务摘要                               |
| `agent_get_logs_status`  | —            | `{ enabled, source }`                 | 日志源路径与可用性                         |
| `agent_read_logs`        | `{ limit? }` | `{ source, content }`                 | 一次性读取尾部日志                         |
| `agent_start_log_stream` | —            | `{ started }`                         | 后台 tail，增量推 `agent/log-stream` Event |
| `agent_stop_log_stream`  | —            | `{ started: false }`                  | 停止 tail                                  |

---

### 企微账号（SQLite）

| Command                        | 参数                             | 返回                      |
| ------------------------------ | -------------------------------- | ------------------------- |
| `wework_list_accounts`         | —                                | `WeworkSavedAccountDto[]` |
| `wework_upsert_account`        | `account: WeworkSavedAccountDto` | `WeworkSavedAccountDto`   |
| `wework_delete_account`        | `id: string`                     | `void`                    |
| `wework_get_active_account_id` | —                                | `string \| null`          |
| `wework_set_active_account_id` | `id: string \| null`             | `void`                    |

`WeworkSavedAccountDto`：`{ id, label, config, createdAt, lastConnectedAt?, weworkUserId? }`  
`config`：`{ weworkExePath?, weworkVersion?, weworkSmart?, weworkInitWaitSeconds? }`

---

## Event 参考

### Rust → 前端（订阅）

| Event                    | `TauriEvent`           | 用途                    |
| ------------------------ | ---------------------- | ----------------------- |
| `session/changed`        | `SessionChanged`       | 语言/会话变更广播       |
| `modal/opened`           | `ModalOpened`          | Modal 窗口已打开        |
| `modal/closed`           | `ModalClosed`          | Modal 窗口已关闭        |
| `modal/open-panel`       | `ModalOpenPanel`       | 通知 Modal 打开指定面板 |
| `agent/stream-chunk`     | `AgentStreamChunk`     | Agent 流式输出增量      |
| `agent/run-finished`     | `AgentRunFinished`     | Agent 单次 run 结束     |
| `agent/log-stream`       | `AgentLogStream`       | 日志 tail 增量          |
| `channel/status-changed` | `ChannelStatusChanged` | 渠道连接状态变更        |

### 前端 → Rust（emit，非 invoke）

| Event        | `TauriEvent` | 用途                                            |
| ------------ | ------------ | ----------------------------------------------- |
| `fe/log`     | `FeLog`      | 前端写 Rust 日志（**推荐**，勿新增 log invoke） |
| `fe/log-req` | `FeLogReq`   | 前端请求链路日志                                |

> `log_fe` / `log_fe_req` 两个 invoke 仍存在，但规范上应优先用 Event。

---

## Rust 内部 API（非 IPC，供后端开发）

以下方法**不能**从前端直接调用，由 `cmd` 或 `events` 委托。

### `context/agent_runtime.rs` — `AgentRuntime`

| 方法                                                                                            | 说明                             |
| ----------------------------------------------------------------------------------------------- | -------------------------------- |
| `initialize(app)`                                                                               | 启动时初始化 runtime             |
| `start_sidecar_deferred()`                                                                      | 延迟启动 Python channel sidecar  |
| `console_state()`                                                                               | 聚合控制台状态                   |
| `send_message(app, message)`                                                                    | 发送消息并注册流式回调           |
| `refresh_skills()`                                                                              | 刷新 skills                      |
| `update_provider(...)` / `clear_provider(...)` / `set_active_chat(...)`                         | Provider 配置                    |
| `session_id()` / `new_session()` / `clear_context()`                                            | Session 管理                     |
| `list_sessions()`                                                                               | Session 索引                     |
| `list_knowledge_files()` / `read_knowledge_file()` / `knowledge_graph()`                        | 知识库                           |
| `upload_knowledge_files()` / `pick_and_upload_knowledge()`                                      | 知识入库                         |
| `list_channels()` / `channel_python_channels_get()` / `channel_python_channels_post()`          | 渠道                             |
| `channel_console_api(path, method, body)`                                                       | 渠道 console API 代理            |
| `channel_reply()` / `channel_process()` / `channel_decorate_text()` / `channel_extract_media()` | 渠道消息处理（Rust 侧）          |
| `logs_status()` / `read_logs()` / `start_log_stream()` / `stop_log_stream()`                    | 日志                             |
| `list_memory_items()` / `read_memory_item()`                                                    | Memory 文件                      |
| `list_task_items()`                                                                             | 定时任务                         |
| `ensure_agent()` / `with_agent_read()` / `with_agent_write()`                                   | Agent 实例访问                   |
| `cancel_request(request_id)`                                                                    | 取消请求（module 级）            |
| `run_agent_message(...)`                                                                        | 后台执行 agent 消息（module 级） |

### `context/session.rs` — 语言会话

| 方法                                                | 说明              |
| --------------------------------------------------- | ----------------- |
| `get_session(app)`                                  | 读取 `AppSession` |
| `set_current_language(app, lang)`                   | 切换语言并广播    |
| `broadcast_session()` / `push_session_to_webview()` | 会话同步          |
| `read_stored_lang()` / `write_stored_lang()`        | 磁盘持久化        |

### `context/license_store.rs` — 授权

| 方法                                 | 说明             |
| ------------------------------------ | ---------------- |
| `snapshot()`                         | 授权快照         |
| `apply_activation_token(app, token)` | 应用激活码       |
| `require_valid()`                    | Command 门禁检查 |

### `context/wework_accounts.rs` — 企微账号

| 方法                                                        | 说明         |
| ----------------------------------------------------------- | ------------ |
| `list_accounts()` / `upsert_account()` / `delete_account()` | CRUD         |
| `get_active_account_id()` / `set_active_account_id()`       | 当前活跃账号 |

### `context/workspace_console.rs` — 工作区文件

| 方法                                                                           | 说明                 |
| ------------------------------------------------------------------------------ | -------------------- |
| `list_session_summaries()`                                                     | Session 索引读写     |
| `list_knowledge_files()` / `read_knowledge_file()` / `build_knowledge_graph()` | 知识库扫描           |
| `list_channels_from_config()`                                                  | 从 config 读渠道列表 |

### `context/channel_python_sidecar.rs` — Python Sidecar

| 方法                                                       | 说明                  |
| ---------------------------------------------------------- | --------------------- |
| `ensure_running()`                                         | 确保 sidecar 进程存活 |
| `channels_get()` / `channels_post()` / `channels_status()` | HTTP 代理             |
| `console_api(path, method, body)`                          | Console API 代理      |

### `utils/knowledge_pick.rs`

| 方法                                           | 说明                                                                 |
| ---------------------------------------------- | -------------------------------------------------------------------- |
| `pick_and_read_supported_knowledge_files(app)` | 原生文件选择器 + 磁盘读取，供 `agent_pick_and_upload_knowledge` 使用 |

---

## 源码位置速查

```
src-tauri/src/
├── lib.rs                 # generate_handler! 注册表
├── cmd/
│   ├── agent.rs           # Agent 控制台 IPC（最大模块）
│   ├── lang.rs
│   ├── session.rs
│   ├── window.rs
│   ├── license.rs
│   ├── log.rs
│   └── wework_accounts.rs
├── context/               # .manage Store，cmd 委托目标
├── utils/                 # 无 Store 工具（knowledge_pick、window 等）
└── events/names.rs        # Event 名常量

packages/shared/src/tauri-bridge/
├── enums/tauri-cmd.ts     # TauriCmd 枚举
├── enums/tauri-event.ts   # TauriEvent 枚举
└── cmd/*.ts               # invokeWrapper 封装
```

---

## 维护说明

新增 Command 时按 [fullstack-ipc.md](./development-rules/fullstack-ipc.md) 清单走完后，在本文件「快速索引」与对应分组中补一行。  
类型变更后运行：

```bash
pnpm run generate:contracts
```
