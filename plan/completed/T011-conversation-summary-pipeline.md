# T011 · Conversation Summary Pipeline

| Field      | Value                 |
| ---------- | --------------------- |
| ID         | T011                  |
| Priority   | P2                    |
| Status     | completed             |
| Depends on | T009, T010            |
| Blocks     | —                     |
| Milestone  | M4 Memory and Profile |

## Goal

长对话自动总结并写入长期记忆，弥补当前仅全量消息堆叠、无压缩策略的问题。

## Scope

1. 触发条件：token 阈值 / 消息条数 / 会话关闭
2. `Summarizer`：调用配置的 chat model 生成摘要
3. 摘要写入 `memory` 源且带来源标记 `conversation_summary`
4. 可选：压缩后保留最近 N 条原始消息在会话上下文

## Acceptance criteria

- [x] 超长会话触发后，`memory_search` 可命中摘要
- [x] 摘要不覆盖原始会话记录（可追溯）
- [x] 配置可关闭自动总结

## Key files

- `src-tauri/src/services/agent/memory/conversation_store.rs`
- `src-tauri/src/services/agent/memory/manager.rs`
- `src-tauri/src/context/agent_runtime/`（会话生命周期钩子）

## Notes

依赖 T009 分库后更易实现独立 summary 任务队列。
