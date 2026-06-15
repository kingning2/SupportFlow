# T007 · Decouple WeWork Hardcoding

| Field      | Value            |
| ---------- | ---------------- |
| ID         | T007             |
| Priority   | P1               |
| Status     | pending          |
| Depends on | T006             |
| Blocks     | T008             |
| Milestone  | M3 Multi-Channel |

## Goal

将企微专用逻辑收敛到 wework 适配器内，配置与路由层变为渠道无关。

## Scope

1. `services/channel/config.rs`：通用字段 + `channel_specific` 扩展块
2. `cmd/wework_accounts.rs` 重命名或拆为 `cmd/channel_accounts.rs` + wework 实现
3. 前端 `apps/wework` 保留风味，但 shared contracts 使用通用 `ChannelAccount`
4. 清理 `create_channel` 中非 wework 的 `ValueError` 硬拒绝，改为 registry 查找

## Acceptance criteria

- [ ] 配置读写对 `channel_type=wework` 与将来 `channel_type=xxx` 同路径
- [ ] grep `wework` 在 `channel_runtime` 核心层显著减少
- [ ] 企微端到端：登录、收件箱、发消息仍可用

## Key files

- `src-tauri/src/services/channel/config.rs`
- `src-tauri/src/cmd/wework_accounts.rs`
- `channel_agent/channel/channel_manager.py`
- `packages/shared/src/channel-core/`

## Notes

可与前端 wework 壳并行；Rust/Python 契约优先。
