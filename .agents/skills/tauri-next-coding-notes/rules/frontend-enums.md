# 前端枚举（禁止魔法字符串）

## Rule

与 IPC、路由、语言、缓存、Redux slice、Modal 面板等相关的**固定字符串**，必须定义在 `src/enums/`，通过 `import { … } from '@/enums'` 引用。

展示给用户的文案仍走 **i18n**（`src-tauri/resources/languages/*.json`），不要放进 enums。

## 目录

| 枚举 | 文件 | 说明 |
|------|------|------|
| `Language` | `language.ts` | 支持的语言；`isLanguage()` 收窄 |
| `LocalCacheKey` | `local-cache-key.ts` | `localStorage` 键 |
| `WindowLabel` | `window-label.ts` | Webview label，与 `tauri.conf.json` / Rust 一致 |
| `TauriEvent` | `tauri-event.ts` | 与 `src-tauri/src/events/names.rs` 一致 |
| `TauriCmd` | `tauri-cmd.ts` | 与 `lib.rs` `generate_handler!` 一致 |
| `ModalPanel` | `modal-panel.ts` | Modal 面板注册名；`isModalPanel()` |
| `FeLogLevel` | `fe-log-level.ts` | 前端写 Rust 日志级别 |
| `ReduxSlice` | `redux-slice.ts` | Redux `createSlice({ name })` |
| `AppRoute` | `app-route.ts` | App Router 路径 |

事件载荷类型在 `src/types/tauri-payloads.ts`；typeshare 契约在 `src/generated/contracts.ts`（勿手改）。

## 扩展清单

**新 Tauri 命令**：`TauriCmd` → `src/cmd/*.ts` 使用 `invokeWrapper(TauriCmd.Xxx)` → Rust `cmd` + `lib.rs`。

**新事件**：Rust `events/names.rs` → `TauriEvent` → 监听处 `tauriOn(TauriEvent.Xxx)`；载荷类型加到 `tauri-payloads.ts`。

**新 Modal 面板**：`ModalPanel` → `components/modal/panels` 实现并注册 `MODAL_PANEL_REGISTRY` → `openModal({ name: ModalPanel.Xxx })`。

**新语言**：`Language` + `cn.json` / `en.json` + `app-config` 的 `supportLanguages`。

## Incorrect

```ts
await setLang('en')
void openModal({ name: 'demo' })
await invokeWrapper('get_app_session')
tauriOn('session/changed', handler)
```

## Correct

```ts
import { Language, ModalPanel, TauriCmd, TauriEvent } from '@/enums'

await setLang(Language.En)
void openModal({ name: ModalPanel.Demo })
await invokeWrapper(TauriCmd.GetAppSession)
tauriOn(TauriEvent.SessionChanged, handler)
```
