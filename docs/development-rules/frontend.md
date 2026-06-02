# 前端编写规范

技术栈：Next.js App Router、React 19、Redux Toolkit、shadcn/ui、Tailwind、GSAP（`apps/full/src/animation/`）。

> 各职位对应的目录说明见 [roles-and-directories.md](./roles-and-directories.md)。  
> 前端 monorepo：`apps/full`（完整控制台）、`apps/wework`、`apps/wechat`；共享库见 `packages/`。

## 目录与职责（完整控制台 `apps/full`）

| 路径                                   | 用途                                                  |
| -------------------------------------- | ----------------------------------------------------- |
| `apps/full/src/app/`                   | 路由与页面壳（`main-window`、`modal-window` 等）      |
| `apps/full/src/components/`            | 可复用 UI；`components/ui/` 为 shadcn 生成件          |
| `apps/full/src/cmd/`                   | Tauri Command 封装（仅 `invokeWrapper` + `TauriCmd`） |
| `apps/full/src/enums/`                 | 固定字符串（命令名、事件名、语言、窗口 label 等）     |
| `apps/full/src/store/`                 | Redux slice（`name` 使用 `ReduxSlice` 枚举）          |
| `apps/full/src/providers/`、`guards/`  | 全局 Provider 与启动/语言守卫                         |
| `apps/full/src/config/`                | 窗口、路由等行为配置                                  |
| `apps/full/src/generated/contracts.ts` | typeshare 生成，**禁止手改**                          |

## `"use client"` 边界

- 使用 Hooks、`useEffect`、浏览器 API、Tauri `invoke`/`listen`、Redux、`useTranslation` 的模块顶部加 `"use client"`。
- 纯展示、无交互的服务端组件可不加；本仓库桌面壳页面多数为 Client Component。

## 禁止魔法字符串

与 IPC、路由、语言、缓存、Modal、Redux slice 相关的字面量必须来自 `@/enums`：

```ts
// ❌
await invoke("get_app_session");
void openModal({ name: "demo" });

// ✅
import { ModalPanel, TauriCmd } from "@/enums";
import { invokeWrapper } from "@/cmd";
await invokeWrapper(TauriCmd.GetAppSession);
void openModal({ name: ModalPanel.Demo });
```

用户可见文案走 **i18n**（`src-tauri/resources/languages/*.json`），不要写进 enums。

## Tauri 调用

| 需求                      | 方式                                                                 |
| ------------------------- | -------------------------------------------------------------------- |
| 需要返回值 / 强类型错误   | `src/cmd/*.ts` → `invokeWrapper(TauriCmd.Xxx)`                       |
| 跨 Webview 广播、单向通知 | `tauriEmit` / `tauriOn` + `TauriEvent`                               |
| 写 Rust 日志              | `src/cmd/log.ts` + `FeLogLevel`（**不要**为日志新增 invoke command） |

禁止在组件内直接 `invoke('字符串')` 或裸 `invokeWrapper('xxx')`。

## 状态

- **跨 Webview 会话等源真相在 Rust `context/`**；前端 Redux 为镜像，经 `TauriEvent.SessionChanged` 或启动时 `get_app_session` 同步。
- 全局 UI 态（语言、modal 蒙层等）进 Redux；纯局部 UI 用 `useState`。
- `useEffect` 依赖数组完整，避免无意义的空依赖数组掩盖遗漏。

## 布局（桌面主窗）

- 根链路透传 `flex`、`min-h-0`、`flex-1`、`overflow-hidden`，避免整窗出现滚动条。
- 需要滚动时，只让**最内层业务区域** `overflow-auto`。
- 标题栏拖拽区遵守 `data-tauri-drag-region` 约定（详见 skill：`titlebar-drag-region.md`）。

## 组件与样式

- 优先复用 `components/ui` 与已有业务组件；新增 shadcn 组件用项目 CLI/技能流程，避免复制粘贴整份 Radix 实现。
- 类名合并用 `cn()`（`clsx` + `tailwind-merge`）。
- 动画逻辑放在 `src/animation/`，组件内只调用封装好的 hook/工具；尊重 `prefers-reduced-motion`。

## TypeScript

- IPC 与 Rust 共享类型优先 `@/generated/contracts`；事件载荷用 `@/types/tauri-payloads`。
- 避免 `any`；语言等枚举用 `isLanguage()` 等类型守卫收窄。
- 提交前：`bun run typecheck`（或 `bun run check`）。

## 错误与日志

- Command 失败由 `invokeWrapper` 抛出 `InvokeError`；页面级用 `error.tsx` / `AppErrorView` 提供恢复路径。
- 异步 `log()` 使用 `void`，不阻塞 UI。
