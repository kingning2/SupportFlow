# T004 · RAG Rerank Layer

| Field      | Value          |
| ---------- | -------------- |
| ID         | T004           |
| Priority   | P0             |
| Status     | completed      |
| Depends on | —              |
| Blocks     | T005           |
| Milestone  | M2 RAG Quality |

## Goal

在现有 hybrid search（向量 + 关键词）之后增加 Rerank，提升 `memory_search` 与知识库召回精度。

## Background

现状：`services/agent/memory/manager.rs` 的 `merge_results` 仅做加权融合，无 Cross-Encoder / LLM rerank。

## Scope

1. 定义 `RerankProvider` trait（输入 query + candidates，输出重排分数）
2. 实现至少一种：OpenAI-compatible rerank API 或本地轻量方案（可先 stub + 配置开关）
3. 在 `DbMemoryManager::search` 流程中：retrieve top-K → rerank → 截断
4. `config.json` 增加 `rerank_provider` / `rerank_model`（可选）

## Acceptance criteria

- [x] 无 rerank 配置时行为与现网一致（向后兼容）
- [x] 开启 rerank 后，同一 query 的 top-3 顺序可变化且可日志对比
- [x] `memory_search` 工具路径走同一套检索栈

## Key files

- `src-tauri/src/services/agent/memory/manager.rs`
- `src-tauri/src/services/agent/memory/embedding.rs`（参考 provider 模式）
- 新建 `src-tauri/src/services/agent/memory/rerank.rs`
- `src-tauri/src/config/models_config.rs`

## Notes

RAG 当前最大缺陷之一；优先于 query rewrite。
