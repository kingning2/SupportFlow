# 开发规范（前后端）

本目录为**与 IDE / AI 工具无关**的编写约定，适用于人工 Code Review、Onboarding、Copilot、Cursor、Claude Code 等场景。

## 文档索引

| 文档                                                       | 适用场景                                             |
| ---------------------------------------------------------- | ---------------------------------------------------- |
| **[roles-and-directories.md](./roles-and-directories.md)** | **按职位看「该改哪个文件夹」**（新人 / 协作必读）    |
| [frontend.md](./frontend.md)                               | Next.js / React / Redux / UI                         |
| [backend-rust.md](./backend-rust.md)                       | Tauri / Rust 分层与实现                              |
| [fullstack-ipc.md](./fullstack-ipc.md)                     | Command / Event / typeshare 契约                     |
| **[rust-api-reference.md](../rust-api-reference.md)**      | **当前 Rust Command / Event / 内部方法速查**         |
| [review-checklist.md](./review-checklist.md)               | 提交 PR 前自检清单                                   |
| **[agent-console.md](../agent-console.md)**                | **SupportFlow 控制台：AI Elements、目录、IPC、扩展** |

## 相关资源

- 架构快照：`.agents/skills/tauri-next-coding-notes/architecture.md`
- 场景化示例：`.agents/skills/tauri-next-coding-notes/examples.md`
- Rust 分层（Cursor 按路径触发）：`.cursor/rules/tauri-rust-layering.mdc`
- Git 提交：`.cursor/rules/git-commit-cn.mdc`（Conventional Commits + 中文简述）

## 原则（全局）

1. **契约优先**：前后端共享的字符串、类型、事件名必须单一来源，改一侧必查清单（见 [fullstack-ipc.md](./fullstack-ipc.md)）。
2. **展示与标识分离**：用户可见文案走 i18n；IPC / 路由 / 窗口 label 等走 `src/enums/` 或 Rust `events/names.rs`。
3. **薄边界、厚实现**：`cmd` 与 `src/cmd` 只做桥接；业务在 Rust `context`/`utils`、前端 `store`/`components`。
4. **最小改动**：只改任务相关文件；不顺手重构、不扩大 scope。
