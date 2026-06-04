# 仓库协作规则

本仓库只保留三类核心协作文档：`Python`、`Rust`、`TypeScript`。
AI 在编写对应语言代码时，必须先遵守对应的“代码规范 + 架构文档 + 文件夹结构文档”。

## 按文件类型强制遵守

- 修改 `src-tauri/channel_agent/**/*.py`、`**/*.py` 时，必须先遵守：
  - `docs/python-coding-rules.md`
  - `docs/python-architecture.md`
  - `docs/python-folder-structure.md`

- 修改 `src-tauri/src/**/*.rs`、`src-tauri/crates/**/*.rs` 时，必须先遵守：
  - `docs/rust-coding-rules.md`
  - `docs/rust-architecture.md`
  - `docs/rust-folder-structure.md`

- 修改 `apps/**/*.ts`、`apps/**/*.tsx`、`packages/**/*.ts`、`packages/**/*.tsx` 时，必须先遵守：
  - `docs/ts-coding-rules.md`
  - `docs/ts-architecture.md`
  - `docs/ts-folder-structure.md`

## 通用要求

1. Python sidecar 目标：只保留 `wx` / `wework` SDK 适配与 `markitdown` 最小骨架，不再承载应用编排层。
2. Rust 目标：拥有桌面应用编排、配置、状态、IPC、AI 工具链与跨端共享业务逻辑。
3. TypeScript 目标：拥有前端界面、状态管理、调用 Rust IPC 的薄桥接，不直接承载后端策略。
4. 若某段旧代码与上述目标冲突，优先朝“Python 更薄、Rust 更重、TS 更清晰”方向整理。
