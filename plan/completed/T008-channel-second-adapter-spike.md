# T008 · Second Channel Adapter Spike

| Field      | Value            |
| ---------- | ---------------- |
| ID         | T008             |
| Priority   | P1               |
| Status     | completed        |
| Depends on | T006, T007       |
| Blocks     | T012             |
| Milestone  | M3 Multi-Channel |

## Goal

用最小第二渠道（Mock）验证适配器契约可扩展。

## Acceptance criteria

- [x] 两个 `channel_type` 可在配置中共存
- [x] inbox 表 `channel` 字段区分来源
- [x] Spike 文档列出实际工作量与风险

## Key files

- `channel_agent/channel/mock/`
- `docs/channel-second-adapter-spike.md`
- `src-tauri/src/context/channel/inbox.rs` — `ingest_sidecar_message`

## Notes

单 sidecar 同时运行多 channel 限制见 T012 ADR。
