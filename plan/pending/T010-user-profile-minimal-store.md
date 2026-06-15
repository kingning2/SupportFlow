# T010 · User Profile Minimal Store

| Field      | Value                 |
| ---------- | --------------------- |
| ID         | T010                  |
| Priority   | P1                    |
| Status     | pending               |
| Depends on | T009                  |
| Blocks     | T011                  |
| Milestone  | M4 Memory and Profile |

## Goal

引入最小用户画像存储，支撑「记住用户偏好」类场景（非完整 CRM）。

## Scope

1. Schema：`user_profiles`（`user_id`、`channel`、`traits` JSON、`updated_at`）
2. `ProfileStore` CRUD + Agent tool `profile_get` / `profile_update`（或扩展现有 memory 工具）
3. 与 channel 用户 ID 映射（wework external_userid 等）
4. 注入 system prompt 片段：可选 `include_profile_in_context`

## Acceptance criteria

- [ ] 同一用户跨会话可读到上次写入的 trait
- [ ] Tool 有权限边界（仅当前会话关联 user）
- [ ] 无 profile 时不影响现有对话

## Key files

- 新建 `src-tauri/src/services/agent/profile/`
- `src-tauri/src/services/agent/tools/memory/`（或新 tools）
- `services/agent/protocol/agent.rs`（context 组装）

## Notes

MVP 用 KV JSON 即可，不做图数据库。
