# 提交 / PR 自检清单

复制到 PR 描述或本地勾选。

## 通用

- [ ] 改动范围与任务一致，无无关重构
- [ ] `bun run check` 通过（format + lint + typecheck）
- [ ] 未提交 `.env`、密钥、本地路径等敏感信息
- [ ] Commit message：`type(scope): 中文简述`（见 `.cursor/rules/git-commit-cn.mdc`）

## 仅前端

- [ ] 无新增魔法字符串（应用 `@/enums`）
- [ ] 用户文案在 i18n JSON，非硬编码中文/英文在组件里
- [ ] Tauri 调用经 `src/cmd`，非裸 `invoke`
- [ ] 主窗/模态布局未引入整页滚动条（`min-h-0` 链完整）
- [ ] 新动画走 `src/animation/`，考虑 `prefers-reduced-motion`

## 仅 Rust

- [ ] 代码落在正确分层（`cmd` / `context` / `utils` / `platform`）
- [ ] 新 command 已注册 `generate_handler!`
- [ ] 公共函数有文档注释
- [ ] `cargo check` / `bun run check:rust` 通过

## 跨前后端（若 touched IPC）

- [ ] 已对照 [fullstack-ipc.md](./fullstack-ipc.md) 完整清单
- [ ] `TauriCmd` / `TauriEvent` / `WindowLabel` 等与 Rust 一致
- [ ] 若改 typeshare：`bun run generate:contracts` 且 `check:contracts` 无意外 diff
- [ ] 在主窗 + 相关 modal 窗手动冒烟

## 可选（大范围功能）

- [ ] 更新 `.agents/skills/tauri-next-coding-notes/examples.md` 或相关 rule 文档
- [ ] 更新 `docs/` 中与行为变更相关的说明
