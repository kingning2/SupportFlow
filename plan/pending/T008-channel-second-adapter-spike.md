# T008 · Second Channel Adapter Spike

| Field      | Value            |
| ---------- | ---------------- |
| ID         | T008             |
| Priority   | P1               |
| Status     | pending          |
| Depends on | T006, T007       |
| Blocks     | —                |
| Milestone  | M3 Multi-Channel |

## Goal

用最小第二渠道（建议 **Webhook / 模拟渠道** 或 **Telegram Bot**）验证适配器契约可扩展。

## Scope

1. 实现 `MockChannel` 或选定真实渠道的 Python adapter + Rust 注册
2. 走通：connect → 收消息 → 写入 `channel_inbox` → Agent 回复 → send
3. 文档记录新增渠道的 checklist（Python + Rust + 配置 + 前端可选）

## Acceptance criteria

- [ ] 两个 `channel_type` 可在配置中共存（即使同时只运行一个 sidecar 实例，见 T012）
- [ ] inbox 表 `channel_type` 字段区分来源
- [ ] Spike 文档列出实际工作量与风险

## Key files

- `channel_agent/channel/`（新 adapter）
- `src-tauri/src/channel_runtime/`
- `src-tauri/src/cmd/channel_inbox.rs`

## Notes

不必做完整产品化；目标是证明「不是只能企微」。
