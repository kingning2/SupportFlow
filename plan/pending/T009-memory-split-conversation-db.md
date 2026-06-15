# T009 · Split Conversation and Memory DB

| Field      | Value                 |
| ---------- | --------------------- |
| ID         | T009                  |
| Priority   | P1                    |
| Status     | pending               |
| Depends on | —                     |
| Blocks     | T010, T011            |
| Milestone  | M4 Memory and Profile |

## Goal

将会话持久化与长期记忆向量索引拆到不同 SQLite，降低耦合与迁移风险。

## Background

现状：`conversation_store` 与 `memory/storage.rs` 共用 `memory/long-term/index.db`。

## Scope

1. 新路径：`{workspace}/conversations/index.db`（或 app_data）
2. 迁移工具：一次性从旧库导出会话表 → 新库（可 CLI `sf migrate-conversations`）
3. `ConversationStore` 只连会话库；`DbMemoryManager` 只连记忆库
4. 更新 `knowledge/ingest` 同步路径说明

## Acceptance criteria

- [ ] 新安装默认分离；旧安装迁移可幂等
- [ ] 会话列表 / 恢复 / memory_search 均正常
- [ ] `docs/project-architecture.md` 数据层描述更新

## Key files

- `src-tauri/src/services/agent/memory/conversation_store.rs`
- `src-tauri/src/services/agent/memory/storage.rs`
- `src-tauri/src/services/agent/knowledge/ingest.rs`

## Notes

与 T002 workflow DB 设计保持一致的分库原则。
