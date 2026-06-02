# 前端编写规范

技术栈：Next.js App Router、React 19、Redux Toolkit、shadcn/ui、Tailwind、GSAP（`packages/ui/src/modal/motion/` 等）。

> 各职位对应的目录说明见 [roles-and-directories.md](./roles-and-directories.md)。  
> 前端 monorepo：`apps/full`（完整控制台）、`apps/wework`、`apps/wechat`；共享库见 `packages/`。

## 目录与职责

| 路径                                     | 用途                                                    |
| ---------------------------------------- | ------------------------------------------------------- |
| `apps/full/src/app/`                     | 完整控制台路由（`main-window`、`modal-window`）         |
| `apps/full/src/components/modal/panels/` | Modal 面板实现与注册（应用私有）                        |
| `apps/full/src/config/`                  | 窗口 label、打开/关闭命令 re-export                     |
| `apps/wework/src/`、`apps/wechat/src/`   | 渠道页、accent、布局（不进 packages）                   |
| `packages/shared/src/tauri-bridge/`      | `invokeWrapper`、`TauriCmd`/`TauriEvent` 枚举、事件辅助 |
| `packages/shared/src/desktop-shell/`     | Redux、Provider、守卫、跨 Webview 同步                  |
| `packages/shared/src/contracts/`         | typeshare 生成与事件载荷类型                            |
| `packages/ui/src/agent-console/`         | 控制台 UI                                               |
| `packages/ui/src/`                       | shadcn 基础件、标题栏、Modal、渠道壳                    |

## `"use client"` 边界

- 使用 Hooks、`useEffect`、浏览器 API、Tauri `invoke`/`listen`、Redux、`useTranslation` 的模块顶部加 `"use client"`。
- 纯展示、无交互的服务端组件可不加；本仓库桌面壳页面多数为 Client Component。

## 禁止魔法字符串

与 IPC、路由、语言、缓存、Modal、Redux slice 相关的字面量必须来自 `@supportflow/shared/tauri-bridge/enums`：

```ts
// ❌
await invoke("get_app_session");
void openModal({ name: "demo" });

// ✅
import { ModalPanel, TauriCmd } from "@supportflow/shared/tauri-bridge/enums";
import { invokeWrapper } from "@supportflow/shared/tauri-bridge/cmd/invoke";
await invokeWrapper(TauriCmd.GetAppSession);
void openModal({ name: ModalPanel.Demo });
```

用户可见文案走 **i18n**（`src-tauri/resources/languages/*.json`），不要写进 enums。

## Tauri 调用

| 需求                      | 方式                                                                          |
| ------------------------- | ----------------------------------------------------------------------------- |
| 需要返回值 / 强类型错误   | `tauri-bridge/cmd/*.ts` → `invokeWrapper(TauriCmd.Xxx)`                       |
| 跨 Webview 广播、单向通知 | `tauriEmit` / `tauriOn` + `TauriEvent`                                        |
| 写 Rust 日志              | `tauri-bridge/cmd/log.ts` + `FeLogLevel`（**不要**为日志新增 invoke command） |

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

- 优先复用 `packages/ui` 与已有业务组件；新增 shadcn 组件用项目 CLI/技能流程，避免复制粘贴整份 Radix 实现。
- 类名合并用 `cn()`（`@supportflow/shared`）。
- 动画逻辑放在 `packages/ui` 动效封装处；尊重 `prefers-reduced-motion`。

## TypeScript

- IPC 与 Rust 共享类型优先 `@supportflow/shared/contracts`；事件载荷用 `contracts/tauri-payloads`。
- 避免 `any`；语言等枚举用 `isLanguage()` 等类型守卫收窄。
- 提交前：`pnpm run typecheck`（或 `pnpm run check`）。

## 错误与日志

- Command 失败由 `invokeWrapper` 抛出 `InvokeError`；页面级用 `error.tsx` / `AppErrorView` 提供恢复路径。
- 异步 `log()` 使用 `void`，不阻塞 UI。
