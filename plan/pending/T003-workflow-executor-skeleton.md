# T003 · Workflow Executor Skeleton

| Field      | Value           |
| ---------- | --------------- |
| ID         | T003            |
| Priority   | P0              |
| Status     | pending         |
| Depends on | T001, T002      |
| Blocks     | —               |
| Milestone  | M1 Workflow MVP |

## Goal

实现可跑通的最小工作流执行器：加载定义 → 按节点执行 → 持久化状态 → 发出事件。

## Scope

1. `WorkflowExecutor::start(run_id, definition)`
2. 节点处理器注册表（至少实现 `agent_reply` 节点，内部调用现有 `AgentBridge::agent_reply`）
3. 失败重试策略（MVP：固定 1 次重试 + 标记 failed）
4. Tauri 事件：`workflow/step_started`、`workflow/step_finished`、`workflow/run_finished`
5. 可选 IPC：`workflow_start`、`workflow_get_run`（薄 cmd 层）

## Acceptance criteria

- [ ] 硬编码 YAML/JSON 定义可跑通 3 步线性流程
- [ ] 中断后可通过 `load_run` 继续或标记 failed
- [ ] 前端或 CLI 能订阅到 step 事件

## Key files

- 新建 `src-tauri/src/services/workflow/executor.rs`
- `src-tauri/src/cmd/`（新 workflow IPC，可选）
- `src-tauri/src/events/`（新事件名）

## Notes

M1 完成标志：有 workflow 而不仅是 agent chat loop。
