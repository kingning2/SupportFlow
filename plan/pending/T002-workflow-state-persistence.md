# T002 · Workflow State Persistence

| Field      | Value           |
| ---------- | --------------- |
| ID         | T002            |
| Priority   | P0              |
| Status     | pending         |
| Depends on | T001            |
| Blocks     | T003            |
| Milestone  | M1 Workflow MVP |

## Goal

工作流运行态可落盘、可恢复，避免进程重启后流程丢失。

## Scope

1. 选择存储：建议独立 SQLite（`{workspace}/workflow/runs.db` 或 app_data_dir）
2. 表设计：`workflow_runs`、`workflow_steps`、`workflow_events`
3. 实现 `WorkflowStore`：create_run / append_event / update_step / load_run
4. 与现有 `conversation_store` 分离，不共用 `memory/long-term/index.db`

## Acceptance criteria

- [ ] Schema + migration 脚本或 `CREATE TABLE IF NOT EXISTS`
- [ ] 单元测试：创建 run → 写入 step → 重启后 load 一致
- [ ] 文档记录 DB 路径与备份策略

## Key files

- 新建 `src-tauri/src/services/workflow/store.rs`
- 参考 `services/agent/memory/conversation_store.rs`（仅参考模式，不合并 DB）

## Notes

避免重复 T009 的问题（会话与记忆共库）。
