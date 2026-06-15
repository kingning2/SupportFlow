# T015 · Production Observability Baseline

| Field      | Value                   |
| ---------- | ----------------------- |
| ID         | T015                    |
| Priority   | P2                      |
| Status     | pending                 |
| Depends on | T003                    |
| Blocks     | —                       |
| Milestone  | M5 Production Hardening |

## Goal

建立上线前最低限度的可观测性：结构化日志、关键指标、错误聚合。

## Scope

1. 统一 trace id：`session_id` / `workflow_run_id` / `channel_message_id` 贯穿日志
2. 指标埋点（MVP）：agent 延迟、tool 失败率、RAG 命中率、sidecar 重启次数
3. 前端 Logs 视图对接新字段（`packages/ui/.../logs.tsx`）
4. 敏感信息脱敏规则文档

## Acceptance criteria

- [ ] 一次完整 agent 回复可在日志中按 trace_id 串联
- [ ] 至少 4 个指标可在 dev 环境查看（日志或简单 counter 文件）
- [ ] `docs/project-architecture.md` 运维小节更新

## Key files

- `src-tauri/src/events/`
- `src-tauri/src/context/agent_runtime/`
- `packages/ui/src/agent-console/views/logs.tsx`

## Notes

不做完整 Prometheus 集成亦可；先本地可调试。
