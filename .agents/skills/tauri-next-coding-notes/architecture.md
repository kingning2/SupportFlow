# Architecture Snapshot

> 按职位查看「主要改哪些文件夹」：[`docs/development-rules/roles-and-directories.md`](../../../docs/development-rules/roles-and-directories.md)

## Monorepo 总览

```
apps/full/          完整控制台（路由、modal panels、窗口配置）
apps/wework/        企微独立应用（渠道页在各自 src/）
apps/wechat/        微信独立应用
packages/shared/    IPC、枚举、Redux、Provider、contracts
packages/ui/        控制台、shadcn、标题栏、Modal、动效
src-tauri/          Rust 桌面端
```

依赖方向：`shared` → `ui` → `apps/*`。渠道私有页面只放在对应 `apps/<channel>/src/`，不进 `packages/`。

## `packages/shared` 职责

- `tauri-bridge/cmd/*`：前端到 Tauri 的命令封装（`invokeWrapper` + `TauriCmd`）。
- `tauri-bridge/enums/*`：固定字符串（命令名、事件名、语言、窗口 label 等）。
- `tauri-bridge/tauri-event.ts`、`cache.ts`：事件 emit/listen、本地缓存。
- `desktop-shell/store/*`：Redux（`app`、`modal` slices，名用 `ReduxSlice` 枚举）。
- `desktop-shell/providers/*`：`StoreProvider`、`TauriEventProvider`。
- `desktop-shell/events/cross-webview-sync.ts`：跨 Webview 会话同步（→ Redux）。
- `desktop-shell/guards/*`、`config/*`：启动、语言、应用配置。
- `contracts/contracts.ts`：typeshare 生成（如 `AppSession`，**勿手改**）。
- `contracts/tauri-payloads.ts`：事件载荷等手写 TS 类型。

## `packages/ui` 职责

- `agent-console/*`：控制台页面、聊天、频道视图等。
- `modal/*`、`title-bar/*`、`app-shell/*`：模态窗、标题栏、渠道壳布局。
- `*.tsx`（根级）：shadcn 基础组件（`button`、`dialog` 等）。
- `modal/motion/*`：GSAP 窗口动效。

## `apps/full` 职责

- `app/*`：App Router（`main-window`、`modal-window`）。
- `components/modal/panels/*`：Modal 面板实现与 `MODAL_PANEL_REGISTRY`（应用私有）。
- `config/windows.ts`：窗口 label 判断、re-export 打开/关闭命令。
- `guards/`、`assets/`：主窗背景守卫、全局 CSS。

## `src-tauri` 职责

- `src/cmd/*`：Rust command 薄入口。
- `src/context/*`：`.manage` 持有的跨 Webview 共享态（会话）。
- `src/utils/*`：无 Store 的通用逻辑（窗口、日志等）。
- `resources/languages/*.json`：i18n 资源。

## 调用链

### Command（invoke）

1. 业务代码调用 `packages/shared/src/tauri-bridge/cmd/*.ts`（`TauriCmd` 枚举）。
2. `invokeWrapper(TauriCmd.Xxx, args)`。
3. Rust command 在 `src-tauri/src/cmd/*.rs`（逻辑在 `utils/` 或 `context/`）。
4. `src-tauri/src/lib.rs` 注册 `generate_handler!`。

### Event（emit / listen）

1. 事件名：前端 `tauri-bridge/enums/tauri-event.ts` ↔ Rust `events/names.rs`（字符串值必须一致）。
2. Rust → 前端：`events/emit.rs`（会话、modal 生命周期等）。
3. 前端 → Rust：`tauriEmit` → `events/handlers/*`（如 `fe/log`）。
4. 全局订阅：`TauriEventProvider` 内 `CrossWebviewSyncSubscriptions`（会话 → Redux）。
5. `lib.rs` 的 `setup` 调用 `events::setup` 注册 listen。

## 前后端契约

- Rust 类型标注 `#[typeshare]` → `pnpm run generate:contracts` 生成 `packages/shared/src/contracts/contracts.ts`。
- IPC 共享类型：`@supportflow/shared/contracts`；事件载荷：`@supportflow/shared/contracts/tauri-payloads`。

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
