# Architecture Snapshot

> 按职位查看「主要改哪些文件夹」：[`docs/development-rules/roles-and-directories.md`](../../../docs/development-rules/roles-and-directories.md)

## 目录职责

- `src/app/*`：页面与路由层（App Router）。
- `src/components/*`：UI 与壳层组件（标题栏、Modal 等）。
- `src/cmd/*`：前端到 Tauri 的命令封装（`invokeWrapper` + `TauriCmd`）。
- `src/enums/*`：前端固定字符串（命令名、事件名、语言、窗口 label 等）。
- `src/types/*`：事件载荷等 TS 类型（如 `tauri-payloads.ts`）。
- `src/config/windows.ts`：主窗 / modal 窗口与打开命令封装。
- `src/events/cross-webview-sync.ts`：跨 Webview 会话同步（→ Redux）。
- `src/providers/tauri-event-provider.tsx`：Tauri emit/listen + `CrossWebviewSyncSubscriptions`。
- `src/store/*`：Redux（`app`、`modal` slices，名用 `ReduxSlice` 枚举）。
- `src/generated/contracts.ts`：typeshare 生成（如 `AppSession`）。
- `src-tauri/src/cmd/*`：Rust command 薄入口。
- `src-tauri/src/context/*`：`.manage` 持有的跨 Webview 共享态（会话）。
- `src-tauri/src/utils/*`：无 Store 的通用逻辑（窗口、日志等）。
- `src-tauri/resources/languages/*.json`：i18n 资源。

## 调用链

### Command（invoke）

1. 前端页面调用 `src/cmd/*.ts`（参数使用 `TauriCmd` 等枚举）。
2. `invokeWrapper(TauriCmd.Xxx, args)`。
3. Rust command 在 `src-tauri/src/cmd/*.rs`（逻辑在 `utils/` 或 `context/`）。
4. `src-tauri/src/lib.rs` 注册 `generate_handler!`。

### Event（emit / listen）

1. 事件名：前端 `src/enums/tauri-event.ts` ↔ Rust `events/names.rs`（字符串值必须一致）。
2. Rust → 前端：`events/emit.rs`（会话、modal 生命周期等）。
3. 前端 → Rust：`tauriEmit` → `events/handlers/*`（如 `fe/log`）。
4. 全局订阅：`TauriEventProvider` 内 `CrossWebviewSyncSubscriptions`（会话 → Redux）。
5. `lib.rs` 的 `setup` 调用 `events::setup` 注册 listen。

## 前后端契约

- Rust 类型标注 `#[typeshare]` → `bun run generate:contracts` 生成 `src/generated/contracts.ts`。
- 前端 IPC 辅助类型优先 `@/generated/contracts`；事件载荷见 `@/types/tauri-payloads`。

## i18n 流程

1. `get_lang` / `get_app_session` 获取当前语言（`Language` 枚举）。
2. `get_language_resource_bundle` 从资源目录读取 JSON。
3. `LanguageGuard` 注入 i18next bundle。
4. 页面通过 `useTranslation(namespace)` 取文案。
5. `set_lang` 写 Rust `context/session` 后广播 `session/changed`。

## 主窗布局原则

- 标题栏高度独立。
- 主内容区优先固定高度下排版，不依赖全局滚动。
- 需要滚动时，只让最内层业务区域滚动。
