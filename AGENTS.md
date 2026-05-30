# Agent / 协作者说明

本仓库为 **Tauri 2 + Next.js** 桌面应用模板。编写或审查代码前请先读：

- **角色 × 目录（各职位改哪里）**：[`docs/development-rules/roles-and-directories.md`](docs/development-rules/roles-and-directories.md)
- **总规范索引**：[`docs/development-rules/README.md`](docs/development-rules/README.md)
- **前端**：[`docs/development-rules/frontend.md`](docs/development-rules/frontend.md)
- **Rust 后端**：[`docs/development-rules/backend-rust.md`](docs/development-rules/backend-rust.md)
- **IPC 契约**：[`docs/development-rules/fullstack-ipc.md`](docs/development-rules/fullstack-ipc.md)
- **Agent 控制台**：[`docs/agent-console.md`](docs/agent-console.md)（AI Elements、SupportFlow 布局、IPC 流式）
- **PR 自检**：[`docs/development-rules/review-checklist.md`](docs/development-rules/review-checklist.md)

## 不可违反的三条

1. 固定字符串 → `src/enums/`（或 Rust `events/names.rs`）；展示文案 → i18n。
2. Tauri：**Command** 走 `invokeWrapper` + `TauriCmd`；**Event** 走 `TauriEvent`；改 IPC 必须走完 [`fullstack-ipc.md`](docs/development-rules/fullstack-ipc.md) 清单。
3. Rust 分层：`cmd` 薄、`context` 存跨 Webview 态、`utils` 无 Store 业务、`platform/` 分 OS。

## 常用命令

```bash
bun run dev          # 仅前端
bun run tauri dev    # 桌面调试
bun run check        # format + lint + typecheck
bun run check:rust   # cargo check
bun run generate:contracts
```

## 深度参考（场景化）

技能包：`.agents/skills/tauri-next-coding-notes/`（含布局、标题栏、i18n、事件总线等细分 rule）。
