# T014 · Multi-Agent Role Model

| Field      | Value                   |
| ---------- | ----------------------- |
| ID         | T014                    |
| Priority   | P2                      |
| Status     | completed               |
| Depends on | T003                    |
| Blocks     | —                       |
| Milestone  | M5 Production Hardening |

## Goal

定义多 Agent 协作的最小模型（角色分工 + 消息路由），而非单 Agent 包打天下。

## Scope

1. 概念：`AgentRole`（planner / executor / reviewer）、`RoleBinding`（model + tools + skills）
2. Workflow 节点 `delegate_to_role` 或子 session spawn
3. 子 session 上下文隔离与结果回传父 run
4. 限制：MVP 不做 Agent 间自由对话，仅主从调用

## Acceptance criteria

- [x] 设计文档 + 类型定义
- [x] 一个 demo workflow：planner 出计划 → executor 调 tool → reviewer 汇总
- [x] 取消/超时在子 session 可传播（子 `request_id` + `timeout_secs`）

## Key files

- `src-tauri/src/services/bridge/agent_bridge.rs`
- `src-tauri/src/context/agent_runtime/`
- `src-tauri/src/services/workflow/`（依赖 T003）
- [`docs/multi-agent-role-model.md`](../../docs/multi-agent-role-model.md)

## Notes

成熟度审计中 multi-agent 为明显缺口；依赖 workflow 骨架。Demo id: `demo-multi-agent`。
