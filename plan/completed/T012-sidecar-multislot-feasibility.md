# T012 · Sidecar Multi-Slot Feasibility

| Field      | Value                   |
| ---------- | ----------------------- |
| ID         | T012                    |
| Priority   | P2                      |
| Status     | completed               |
| Depends on | T008                    |
| Blocks     | —                       |
| Milestone  | M5 Production Hardening |

## Goal

评估并实现（或明确拒绝）`ProcessHub` 多 sidecar 实例，以支持多 channel 同时在线。

## Background

现状：`context/process_hub.rs` 单 channel slot；`plan/rust-sidecar-async-ipc-architecture.md` 有异步 IPC 草案。

## Scope

1. 方案对比：多进程 sidecar vs 单进程多 channel 路由
2. 若做多进程：`ProcessHub` 改为 `HashMap<channel_id, SidecarHandle>`
3. 健康检查、重启、端口/stdio 隔离
4. 更新架构文档与限制说明

## Acceptance criteria

- [x] ADR 文档：选定方案 + 不做时的产品限制
- [ ] 若实现：两个 mock channel 同时 RPC 无串线（**未实现 — ADR 推迟多进程**）
- [x] 若不做：T008 文档标明「单活跃渠道」限制

## Key files

- `src-tauri/src/context/process_hub.rs`
- `src-tauri/src/python/sidecar/`
- `plan/rust-sidecar-async-ipc-architecture.md`
- [`docs/sidecar-multislot-adr.md`](../../docs/sidecar-multislot-adr.md)

## Notes

可与 T008 spike 合并验证；本任务偏架构决策。**Outcome:** 方案 C（单活跃渠道）。
