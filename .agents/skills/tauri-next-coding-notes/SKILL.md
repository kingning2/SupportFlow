---
name: tauri-next-coding-notes
description: Enforces architecture rules and reusable coding patterns for this Tauri + Next.js desktop app. Use when implementing Tauri commands or events, frontend pages/components, i18n copy, Redux logic, title bar behavior, modal windows, or main-window layout.
user-invocable: true
---

# Tauri + Next 编码规范

本技能用于本仓库日常开发（**桌面模板**，无内置业务域）。前端为 **Turborepo monorepo**（`apps/*` + `packages/*`），优先遵守已存在目录结构与命名方式。

## 关键上下文

- 前端：Next App Router + React + Redux Toolkit + shadcn/ui；完整控制台在 `apps/full`，共享逻辑在 `packages/shared` 与 `packages/ui`。
- **固定字符串**：业务相关字面量统一放 `packages/shared/src/tauri-bridge/enums/`（见 [frontend-enums.md](./rules/frontend-enums.md)），通过 `@supportflow/shared/tauri-bridge/enums` 引用。
- **Command 桥接**：`packages/shared/src/tauri-bridge/cmd/invoke.ts` 的 `invokeWrapper` + `TauriCmd` 枚举。
- **Event 桥接**：`src-tauri/src/events/` ↔ `tauri-bridge/enums/tauri-event.ts` + `tauri-bridge/tauri-event.ts` + `desktop-shell/providers/tauri-event-provider.tsx`。
- **跨 Webview 同步**：`desktop-shell/events/cross-webview-sync.ts`（会话 → Redux）。
- **窗口 / Modal**：`apps/full/src/config/windows.ts`（re-export）；`useModalWindow` 在 `@supportflow/ui/modal`；面板注册在 `apps/full/src/components/modal/panels/`。
- Rust **`context/`**：`.manage` 会话；**`utils/`**：无 Store 的通用逻辑；**`cmd/`**：薄 command。
- Rust 入口：`src-tauri/src/lib.rs`（`generate_handler!` + `events::setup`）。
- i18n：`react-i18next` + `src-tauri/resources/languages/*.json`（展示文案走 i18n，不进 enums）。
- 主窗：`TitleBar` + `ContentContainer`，重点防止滚动条。

## Critical Rules

1. **前端枚举与禁止魔法字符串** → [frontend-enums.md](./rules/frontend-enums.md)
2. **Tauri 命令链路完整性** → [tauri-command-chain.md](./rules/tauri-command-chain.md)
3. **Tauri 事件总线** → [tauri-events.md](./rules/tauri-events.md)
4. **i18n 与文案规范** → [i18n-and-copy.md](./rules/i18n-and-copy.md)
5. **主窗布局与滚动控制** → [layout-and-scroll.md](./rules/layout-and-scroll.md)
6. **标题栏拖拽与交互区域** → [titlebar-drag-region.md](./rules/titlebar-drag-region.md)
7. **状态与副作用边界** → [state-and-effects.md](./rules/state-and-effects.md)
8. **日志与错误处理** → [logging-and-errors.md](./rules/logging-and-errors.md)
9. **提交前检查** → [preflight-checks.md](./rules/preflight-checks.md)
10. **Rust 分层** → 仓库 `.cursor/rules/tauri-rust-layering.mdc`（`cmd` / `context` / `utils`）

## 快速模式（常用）

```tsx
// 页面文本走 i18n；固定标识走 shared enums
import { Language, ModalPanel } from "@supportflow/shared/tauri-bridge/enums";

const { t } = useTranslation("home");
<h1>{t("title")}</h1>;
```

```tsx
// 主窗根布局：持续传递 min-h-0 / flex-1 / overflow-hidden
<ContentContainer className="flex flex-col overflow-hidden">{children}</ContentContainer>
```

```rust
// 新增 tauri command 后，必须进入 lib.rs handler
.invoke_handler(tauri::generate_handler![
  // ...
  cmd::xxx::your_command,
])
```

```tsx
// 打开 modal（Rust command，非 WebviewWindow API）
import { ModalPanel } from "@supportflow/shared/tauri-bridge/enums";
import { useModalWindow } from "@supportflow/ui/modal";

const { openModal } = useModalWindow();
await openModal({ name: ModalPanel.Demo, width: 480, height: 360 });
```

```rust
// Rust → 前端：走 events::emit，勿在 utils 里硬编码事件名
crate::events::session_changed_all(app, &session)?;
```

## 参考资料

- **通用开发规范（IDE 无关）**：仓库根目录 [`docs/development-rules/README.md`](../../../docs/development-rules/README.md)
- 架构与目录： [architecture.md](./architecture.md)
- 场景化片段： [examples.md](./examples.md)
- Apps / Packages 说明： [`apps/README.md`](../../../apps/README.md)、[`packages/README.md`](../../../packages/README.md)
