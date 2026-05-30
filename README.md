# Tauri + Next.js 桌面模板

Tauri 2 承载壳层与系统能力，Next.js App Router 负责 UI；Rust 与前端通过 **Command（invoke）** 与 **Event（emit / listen）** 协作。

![主窗口预览](./public/main.png)

## 架构概览

| 能力         | 实现要点                                                                                    |
| ------------ | ------------------------------------------------------------------------------------------- |
| **主窗**     | 无边框透明窗体 + CSS `--window-radius` 裁切                                                 |
| **Modal**    | Rust `WebviewWindowBuilder` 创建的独立子窗口（`WindowLabel.Modal`），可预热、复用 hide/show |
| **跨窗同步** | 主窗与 modal 各为独立 Webview，通过 Event + Redux 同步语言会话与蒙层状态                    |
| **IPC**      | Command 走 `invoke`；Event 走 `emit` / `listen`                                             |

Modal 打开链路：`src/config/windows.ts` → `src/cmd/window.ts` → `src-tauri/src/cmd/window.rs` → `utils/window.rs`。

更细的目录与约定见 [`.agents/skills/tauri-next-coding-notes/architecture.md`](./.agents/skills/tauri-next-coding-notes/architecture.md)。

## 前端约定：禁止魔法字符串

业务相关的固定字符串（语言、窗口 label、Tauri 命令/事件名、Modal 面板名、缓存键、路由路径等）**不要**在组件或 `cmd` 里直接写 `"cn"`、`"get_lang"`、`"modal/opened"` 这类字面量，统一放在 **`src/enums/`**，通过枚举引用。

```ts
import { Language, ModalPanel, TauriCmd, TauriEvent, WindowLabel } from "@/enums";

// ✅
await setLang(Language.En);
void openModal({ name: ModalPanel.Demo, title: "…" });
await invokeWrapper(TauriCmd.GetAppSession);
tauriOn(TauriEvent.SessionChanged, handler);

// ❌
await setLang("en");
void openModal({ name: "demo" });
await invokeWrapper("get_app_session");
```

事件载荷类型在 `src/types/tauri-payloads.ts`；与 Rust 对齐的契约类型在 `src/generated/contracts.ts`（typeshare 生成，勿手改）。

### `src/enums/` 一览

| 枚举            | 文件                 | 典型用途                                                      |
| --------------- | -------------------- | ------------------------------------------------------------- |
| `Language`      | `language.ts`        | 当前语言、`isLanguage()` 收窄                                 |
| `LocalCacheKey` | `local-cache-key.ts` | `localStorage` 键名                                           |
| `WindowLabel`   | `window-label.ts`    | Webview 窗口 label（与 `tauri.conf.json` 一致）               |
| `TauriEvent`    | `tauri-event.ts`     | `listen` / `tauriOn` 事件名（与 Rust `events/names.rs` 一致） |
| `TauriCmd`      | `tauri-cmd.ts`       | `invokeWrapper` 命令名（与 `lib.rs` handler 一致）            |
| `ModalPanel`    | `modal-panel.ts`     | Modal 面板注册名、`isModalPanel()`                            |
| `FeLogLevel`    | `fe-log-level.ts`    | 前端写 Rust 日志的级别                                        |
| `ReduxSlice`    | `redux-slice.ts`     | Redux slice `name`                                            |
| `AppRoute`      | `app-route.ts`       | App Router 路径（如 modal 子窗 URL）                          |

新增枚举值时：在对应文件增加成员 → 在注册表/配置处改用该成员 → 若涉及 Rust，同步改 `src-tauri` 侧常量并保持字符串值一致。

## 技术栈

| 层级   | 技术                                                   |
| ------ | ------------------------------------------------------ |
| 桌面壳 | Tauri 2                                                |
| 前端   | Next.js 16、React 19、TypeScript                       |
| 状态   | Redux Toolkit                                          |
| UI     | shadcn/ui、Tailwind CSS 4                              |
| 动画   | GSAP（Modal 进出场）                                   |
| 国际化 | react-i18next + `src-tauri/resources/languages/*.json` |
| 契约   | typeshare → `src/generated/contracts.ts`               |

## 环境要求

- [Bun](https://bun.sh/)
- [Rust](https://www.rust-lang.org/tools/install)
- Windows / macOS 桌面构建环境

## 常用命令

```bash
bun run dev          # 仅前端
bun run tauri dev    # 桌面开发
bun run check        # Prettier + ESLint + tsc
bun run check:rust   # cargo check
bun run generate:contracts
```

## 项目结构（简表）

```
src/
  app/                  # 主窗 / modal 路由
  cmd/                  # Tauri invoke 封装（参数使用 TauriCmd 等枚举）
  components/           # 标题栏、Modal、shadcn UI
  config/               # windows、app-config、i18n 初始化
  enums/                # 前端固定字符串（命令、事件、语言、面板等）
  events/               # cross-webview-sync（会话）
  generated/            # typeshare 生成
  store/                # app、modal slices（ReduxSlice 枚举）
  types/                # 事件载荷等 TS 类型

src-tauri/src/
  cmd/                  # lang、session、window、log
  context/              # SessionStore
  events/               # 事件总线（names 与前端 TauriEvent 对齐）
  utils/                # window、log
```

## 从这里开始扩展

1. 在 `src/app/main-window/page.tsx` 替换首页内容。
2. 在 `src/enums/modal-panel.ts` 增加 `ModalPanel` 成员，在 `src/components/modal/panels/` 实现组件并写入 `MODAL_PANEL_REGISTRY`，通过 `openModal({ name: ModalPanel.YourPanel })` 打开。
3. 在 `src-tauri/resources/languages/` 增加文案命名空间（展示文案仍走 i18n，不属于 enums）。
4. 新增 Tauri command：Rust `cmd/*.rs` + `lib.rs` → 前端 `src/enums/tauri-cmd.ts` + `src/cmd/*.ts` 使用 `TauriCmd`。
5. 新增跨窗事件：Rust `events/names.rs` → 前端 `src/enums/tauri-event.ts`，监听处使用 `TauriEvent`。

## Git 提交规范

Conventional Commits + 中文简述，Husky 校验。详见 [`.cursor/rules/git-commit-cn.mdc`](./.cursor/rules/git-commit-cn.mdc)。

## 相关文档

- [Tauri 2](https://v2.tauri.app/)
- [Next.js](https://nextjs.org/docs)
- 编码技能：[`.agents/skills/tauri-next-coding-notes/SKILL.md`](./.agents/skills/tauri-next-coding-notes/SKILL.md)
