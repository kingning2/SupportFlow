# T006 · Channel Adapter Contract

| Field      | Value            |
| ---------- | ---------------- |
| ID         | T006             |
| Priority   | P0               |
| Status     | completed        |
| Depends on | —                |
| Blocks     | T007, T008       |
| Milestone  | M3 Multi-Channel |

## Goal

提炼统一 Channel 适配器契约，让 Rust / Python 两侧对「渠道能力」有同一套抽象。

## Scope

1. 文档化 `ChannelAdapter` 能力矩阵：connect / disconnect / list_conversations / send / on_message / health
2. Rust：`ChannelBridge` 与 `channel_runtime` 对齐，明确错误码与事件 payload
3. Python：`channel_agent/channel/` 下抽象基类 + wework 实现 implements
4. 配置层：`channel_type` 枚举 + 每类型 `config_schema`（JSON Schema 或现有 field defs）

## Acceptance criteria

- [x] `docs/` 或 `plan/completed/` 附契约表（Rust trait ↔ Python 类 ↔ IPC 方法）
- [x] 新增第二类型时无需改 `ProcessHub` 核心逻辑（仅注册）
- [x] 现有 wework 行为无回归

## Key files

- `docs/channel-adapter-contract.md` — 契约表与扩展指南
- `src-tauri/src/services/channel/registry.rs` — 渠道注册表与 config schema
- `src-tauri/src/services/channel/contract.rs` — phase / error code / IPC 常量
- `channel_agent/channel/adapter.py` — Python `ChannelAdapter` ABC
- `channel_agent/channel/registry.py` — Python 工厂注册

## Notes

T007 将把 catalog / config 中残余 wework 硬编码迁入注册表扩展点。
