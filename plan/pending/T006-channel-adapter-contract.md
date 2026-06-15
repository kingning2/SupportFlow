# T006 · Channel Adapter Contract

| Field      | Value            |
| ---------- | ---------------- |
| ID         | T006             |
| Priority   | P0               |
| Status     | pending          |
| Depends on | —                |
| Blocks     | T007, T008       |
| Milestone  | M3 Multi-Channel |

## Goal

提炼统一 Channel 适配器契约，让 Rust / Python 两侧对「渠道能力」有同一套抽象。

## Background

现状：`channel_runtime` 有 `ChatChannel` trait，但 `channel_manager.py` 的 `create_channel` 仅接受 wework；`services/channel/config.rs` 字段定义企微专用。

## Scope

1. 文档化 `ChannelAdapter` 能力矩阵：connect / disconnect / list_conversations / send / on_message / health
2. Rust：`ChannelBridge` 与 `channel_runtime` 对齐，明确错误码与事件 payload
3. Python：`channel_agent/channel/` 下抽象基类 + wework 实现 implements
4. 配置层：`channel_type` 枚举 + 每类型 `config_schema`（JSON Schema 或现有 field defs）

## Acceptance criteria

- [ ] `docs/` 或 `plan/completed/` 附契约表（Rust trait ↔ Python 类 ↔ IPC 方法）
- [ ] 新增第二类型时无需改 `ProcessHub` 核心逻辑（仅注册）
- [ ] 现有 wework 行为无回归

## Key files

- `src-tauri/src/channel_runtime/mod.rs`
- `src-tauri/src/context/channel/`
- `channel_agent/channel/channel_manager.py`
- `packages/shared/src/contracts/contracts.ts`（前端类型同步）

## Notes

M3 前置；不做此步直接加第二渠道会复制粘贴。
