# Tauri Event Bus

## 何时用 Event，何时用 Command

| 场景 | 选用 |
|------|------|
| 需要返回值 / `Result` 错误 | **Command** + `invokeWrapper(TauriCmd.…)` |
| 跨 Webview 广播（会话、modal 蒙层） | **Event**（Rust `emit` / `emit_to`） |
| 前端 → Rust 单向（如写日志） | **Event**（`tauriEmit` + Rust `listen`） |
| 全局 Redux 同步（会话） | **`CrossWebviewSyncSubscriptions`**（`desktop-shell/events/cross-webview-sync.ts`） |

## 事件名（双端同步）

Rust `src-tauri/src/events/names.rs` 与前端 `packages/shared/src/tauri-bridge/enums/tauri-event.ts`（`TauriEvent`）**字符串值必须一致**。

当前约定：

- Rust → 前端：`TauriEvent.SessionChanged`、`TauriEvent.ModalOpened`、`TauriEvent.ModalClosed`、`TauriEvent.ModalOpenPanel`
- 前端 → Rust：`TauriEvent.FeLog`、`TauriEvent.FeLogReq`（新前端事件建议 `fe/<domain>/<action>`）

主窗 label：`WindowLabel.Main`（与 `tauri.conf.json` 一致）。

## Rust 侧改动清单

1. 在 `events/names.rs` 增加常量
2. 在 `events/payloads.rs` 定义载荷（需导出到 TS 时加 `#[typeshare]`，并 `pnpm run generate:contracts`）
3. **Rust → 前端**：在 `events/emit.rs` 增加函数，从 `context` / `cmd` 调用，不要散落 `app.emit_to`
4. **前端 → Rust**：在 `events/handlers/` 增加 handler，并在 `register` 中注册
5. `lib.rs` 的 `setup` 已调用 `events::setup`，一般无需再改

## 前端侧改动清单

1. 在 `tauri-bridge/enums/tauri-event.ts` 增加 `TauriEvent` 成员（与 Rust 同值）
2. 载荷类型加到 `contracts/tauri-payloads.ts`（或 `#[typeshare]` 生成）
3. 发送：`tauriEmit(TauriEvent.Xxx, payload)` 或 `useTauriEventApi().emit`
4. 全局订阅：扩展 `desktop-shell/events/cross-webview-sync.ts`，或页面内 `tauriOn(TauriEvent.Xxx, …)`
5. 根布局已包 `TauriEventProvider`（`apps/full/src/app/layout.tsx` 或渠道 app 的 `layout.tsx`）

## Incorrect

```ts
// ❌ 日志再走 invoke command
await invoke('log_fe', { event: 'info', msg })
```

```ts
// ❌ 硬编码事件名字符串
tauriOn('session/changed', handler)
```

```rust
// ❌ 在 utils 里直接写死事件名字符串
app.emit_to("main", "session/changed", &session)?;
```

## Correct

```ts
import { FeLogLevel, TauriEvent } from "@supportflow/shared/tauri-bridge/enums";
import { tauriEmit } from "@supportflow/shared/tauri-bridge/tauri-event";

await tauriEmit(TauriEvent.FeLog, { level: FeLogLevel.Info, msg: "ready" });
```

```ts
import { TauriEvent } from "@supportflow/shared/tauri-bridge/enums";
import { tauriOn } from "@supportflow/shared/tauri-bridge/tauri-event";

tauriOn(TauriEvent.SessionChanged, (event) => { /* … */ });
```

```rust
crate::events::session_changed_to(app, webview_label, &session)?;
```
