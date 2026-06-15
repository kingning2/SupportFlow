# T001 · Workflow Runtime MVP Design

| Field      | Value           |
| ---------- | --------------- |
| ID         | T001            |
| Priority   | P0              |
| Status     | completed       |
| Depends on | —               |
| Blocks     | T002, T003      |
| Milestone  | M1 Workflow MVP |

## Goal

定义「工作流运行时」最小可行边界，补齐当前仅有 Agent 对话循环、缺少业务流程状态机的缺口。

## Background

现状：`context/agent_runtime` + `services/agent/protocol/agent.rs` 支持单轮/多轮 tool loop，但无节点图、无状态迁移、无人工接管节点。

## Scope

1. 写出 workflow 领域模型（Run / Step / Node / Transition / Context）
2. 明确与现有 `Agent::run_stream` 的关系：workflow 编排 Agent，而非替代 rig
3. 定义首批节点类型：`agent_reply`、`tool_call`、`human_andsign`、`branch`、`delay`
4. 确定持久化落点（新模块路径建议：`src-tauri/src/services/workflow/`）

## Out of scope

- 可视化编排 UI
- 分布式调度

## Acceptance criteria

- [x] `plan/` 或 `docs/` 下有 workflow MVP 设计说明（含状态机图）
- [x] Rust 类型草稿（`WorkflowDefinition`、`WorkflowRun`、`NodeState`）已评审
- [x] 与 `AgentRuntime` 集成点列出（入口 IPC、事件名）

## Key files (expected touch)

- `src-tauri/src/context/agent_runtime.rs`
- `src-tauri/src/services/agent/protocol/agent.rs`
- 新建 `src-tauri/src/services/workflow/`（设计阶段可先仅文档）

## Notes

对标 Coze/CowAgent 的「流程编排」能力；MVP 先支持线性 + 条件分支即可。
