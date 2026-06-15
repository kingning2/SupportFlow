# T013 · Skills Version and Parameters

| Field      | Value                   |
| ---------- | ----------------------- |
| ID         | T013                    |
| Priority   | P2                      |
| Status     | pending                 |
| Depends on | —                       |
| Blocks     | —                       |
| Milestone  | M5 Production Hardening |

## Goal

将 Skills 从「静态 prompt 包」升级为可版本化、可带参数的技能单元。

## Background

现状：`skills/loader.rs` 读 `SKILL.md` frontmatter；`installer.rs` 支持动态安装；无 semver、无参数 schema。

## Scope

1. Frontmatter 扩展：`version`、`parameters`（JSON Schema）
2. `SkillRegistry` 按 name+version 解析；冲突策略（最新 / 锁定）
3. 运行时注入：workflow 或 agent 启动时传入 skill 参数
4. bundled skills 与 user skills 目录版本并存策略

## Acceptance criteria

- [ ] 同一 skill 两个版本可共存，agent 可指定版本
- [ ] 参数校验失败有明确错误
- [ ] 现有 5 个 bundled skills 迁移 frontmatter 不破坏加载

## Key files

- `src-tauri/src/services/agent/skills/loader.rs`
- `src-tauri/src/services/agent/skills/frontmatter.rs`
- `src-tauri/resources/skills/`

## Notes

对标 Coze「技能」可配置化；与 T003 workflow 节点可联动。
