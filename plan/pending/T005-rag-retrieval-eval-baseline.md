# T005 · RAG Retrieval Eval Baseline

| Field      | Value          |
| ---------- | -------------- |
| ID         | T005           |
| Priority   | P1             |
| Status     | pending        |
| Depends on | T004           |
| Blocks     | —              |
| Milestone  | M2 RAG Quality |

## Goal

建立可重复的检索评测基线，避免 RAG 改动凭感觉。

## Scope

1. 准备 `tests/fixtures/rag-eval/` 或 `workspace` 样例：query + 期望命中 path/line
2. 实现 CLI 子命令或测试：`sf rag-eval --workspace ...`
3. 指标：Recall@5、MRR@5（MVP 即可）
4. CI 可选：仅 `--features` 下跑，不阻塞默认 build

## Acceptance criteria

- [ ] 至少 10 条标注样例
- [ ] 一条命令输出 before/after rerank 对比表
- [ ] README 或 plan 内记录如何新增样例

## Key files

- `src-tauri/src/cli/commands/`（新 `rag_eval.rs` 或测试模块）
- `services/agent/memory/manager.rs`

## Notes

为后续 query rewrite、多路召回提供回归护栏。
