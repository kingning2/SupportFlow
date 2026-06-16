# Multi-Agent Role Model (MVP)

Design for T014: minimal role-based delegation on top of the T003 workflow executor.

## Goals

- Separate **planning**, **execution**, and **review** into distinct agent invocations
- Reuse existing `AgentBridge` / `Agent` stack — no peer-to-peer agent chat
- Parent workflow run owns context; child steps write outputs back to `WorkflowContext.vars`

## Non-goals (MVP)

- Free-form multi-agent conversation
- Per-role separate model endpoints (optional `RoleBinding.model` reserved)
- Dynamic agent discovery or autonomous hand-offs

## Concepts

### `AgentRole`

| Role       | Responsibility                                     |
| ---------- | -------------------------------------------------- |
| `planner`  | Decompose user input into steps; no tool execution |
| `executor` | Run tools per plan; produce artifacts / results    |
| `reviewer` | Summarize, validate, produce user-facing reply     |

Defined in `src-tauri/src/services/agent/roles.rs`.

### `RoleBinding`

Maps a role to:

- `model` (optional override)
- `tools` allow-list (advisory for executor)
- `skills` allow-list
- `system_prompt_suffix` injected into the delegated prompt

### Workflow integration

- New node kind: `delegate_to_role` (`NodeKind::DelegateToRole`)
- Config: `DelegateToRoleNodeConfig` — `role`, `prompt_template`, `binding`, `output_key`, `timeout_secs`
- Executor spawns an isolated agent reply via `agent_reply` with:
  - `workflow_run_id` + `agent_role` in `Context` (for trace correlation)
  - Sub-session keyed by `wf-{run_id}-{node_id}` request id

### Demo workflow

`demo-multi-agent` in `services/workflow/definitions.rs`:

```text
plan (planner) → execute (executor) → review (reviewer)
```

Builtin id: `demo-multi-agent`.

## Context isolation

| Layer            | Isolation                                                                                                                   |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Conversation DB  | Sub-request uses unique `request_id`; optional separate `session_id` per role (future)                                      |
| Workflow vars    | Shared parent `WorkflowContext`; each node writes `output_key`                                                              |
| Cancel / timeout | `timeout_secs` on node config; cancel registry keyed by sub `request_id` propagates from parent run cancel (T003 follow-up) |

## Trace correlation

Delegated steps set `workflow_run_id` on channel context. Logs use `trace_id = wf-{run_id}` (see `utils/trace.rs`).

## Related

- Workflow types: `src-tauri/src/services/workflow/types.rs`
- Executor handler: `handle_delegate_to_role` in `executor.rs`
- Sidecar / channel limits: [`sidecar-multislot-adr.md`](./sidecar-multislot-adr.md)
