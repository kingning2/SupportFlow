# RAG 检索评测样例

本目录为 `sf rag-eval` 与 `tests/rag_eval.rs` 共用的标注集。

## 目录结构

```
rag-eval/
  cases.json          # 标注 query + 期望命中 path
  workspace/          # 最小 workspace（MEMORY.md / memory / knowledge）
  README.md
```

## 运行

```bash
cargo run --bin sf -- rag-eval
# 或指定路径
cargo run --bin sf -- rag-eval --fixtures tests/fixtures/rag-eval --workspace tests/fixtures/rag-eval/workspace
```

输出 hybrid vs +rerank 的 Recall@5 / MRR@5 对比表。

## 新增样例

1. 在 `workspace/` 写入或更新 markdown（MEMORY.md、`memory/*.md`、`knowledge/*.md`）
2. 在 `cases.json` 的 `cases` 数组追加一条：
   ```json
   {
     "id": "unique-slug",
     "query": "用户可能发出的检索词",
     "relevant_paths": ["MEMORY.md"]
   }
   ```
3. `relevant_paths` 使用 workspace 内相对路径（`/` 分隔）
4. 运行 `sf rag-eval` 验证 Recall/MRR

## 指标说明

- **Recall@K**：至少一条 relevant 出现在 top-K 的 query 占比
- **MRR@K**：首个 relevant 命中位置的倒数均值（未命中计 0）
