# 全栈 IPC 契约规范

本应用是 **Tauri 桌面壳 + Next 前端**，没有独立 HTTP API；前后端边界即 **Command（invoke）** 与 **Event（emit/listen）**。

## 何时用 Command vs Event

| 场景                                    | 选用                              |
| --------------------------------------- | --------------------------------- |
| 需要返回值、错误处理、启动时拉取状态    | **Command**                       |
| 多 Webview 广播、进度推送、生命周期通知 | **Event**                         |
| 前端写 Rust 日志                        | **Event**（`FeLog` / `FeLogReq`） |

## 新增 Command 清单

按顺序勾选，缺一项即视为未完成：

- [ ] `src-tauri/src/cmd/<module>.rs` — `#[tauri::command]` 实现
- [ ] `src-tauri/src/cmd/mod.rs` — `mod` 导出
- [ ] `src-tauri/src/lib.rs` — `generate_handler!` 注册
- [ ] `src/enums/tauri-cmd.ts` — `TauriCmd` 新成员（字符串 = Rust 命令名）
- [ ] `src/cmd/<name>.ts` — `invokeWrapper(TauriCmd.Xxx, …)` 封装
- [ ] 若参数/返回为共享 DTO：`#[typeshare]` + `pnpm run generate:contracts`
- [ ] 手动验证：主窗与相关 modal 窗均能调用

## 新增 Event 清单

- [ ] `src-tauri/src/events/names.rs` — 常量定义
- [ ] `src/enums/tauri-event.ts` — 同名枚举值
- [ ] Rust：`events/emit.rs` 或 handler 内发射/监听
- [ ] `src/types/tauri-payloads.ts` — 载荷类型（若需要）
- [ ] 前端：`tauriOn` / `TauriEventProvider` / `cross-webview-sync` 等订阅点
- [ ] `lib.rs` → `events::setup` 已注册 listen（前端 → Rust 时）

## 新增 Modal 面板清单

- [ ] `src/enums/modal-panel.ts` — `ModalPanel` 成员
- [ ] `src/components/modal/panels/` — 面板实现
- [ ] `MODAL_PANEL_REGISTRY` 注册
- [ ] 打开时使用 `openModal({ name: ModalPanel.Xxx })`（走 Rust 窗口 command，非直接 `WebviewWindow` API）

## 新增语言清单

- [ ] `src/enums/language.ts` — `Language` 成员
- [ ] `src-tauri/resources/languages/<code>.json`
- [ ] `app-config` 的 `supportLanguages`
- [ ] `set_lang` / 会话广播链路可验证

## 命名一致性（易错点）

| 概念           | Rust                     | 前端                                         |
| -------------- | ------------------------ | -------------------------------------------- |
| Command 名     | `get_app_session`        | `TauriCmd.GetAppSession = 'get_app_session'` |
| Event 名       | `events/names.rs`        | `TauriEvent`                                 |
| 窗口 label     | `tauri.conf.json` / Rust | `WindowLabel`                                |
| Redux slice 名 | —                        | `ReduxSlice`                                 |

**禁止**在三处各写一份不同拼写的字符串。

## 契约变更工作流

1. 先改 Rust 类型并加/改 `#[typeshare]`（如适用）。
2. 运行 `pnpm run generate:contracts`，将 `contracts.ts` diff 一并提交。
3. 再改前端 `cmd` 封装与调用方类型。
4. PR 说明中列出触达的清单项，便于 Reviewer 对照。
