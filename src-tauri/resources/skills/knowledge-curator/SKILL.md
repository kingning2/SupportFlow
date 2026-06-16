---
name: knowledge-curator
version: 1.0.0
description: 知识库整理：归类文档、提炼摘要、维护交叉引用与检索友好标题。
---

# 知识库整理

帮助维护和优化工作区 `knowledge/` 下的 Markdown 知识页。

## 适用场景

- 新文档入库后的分类与命名
- 合并重复内容、更新过期说明
- 为客服/智能体补充可检索的摘要

## 工作流

1. 用 `read` / `memory_get` 阅读目标文档
2. 检查标题层级、frontmatter、内部链接是否一致
3. 提炼 3～5 条要点摘要，写入文首或索引页
4. 相关文档之间补充 `[[wiki-link]]` 或 Markdown 链接
5. 大改动先用 `edit` 小步修改，避免一次性覆盖

## 命名建议

- 文件名用小写连字符：`product-pricing.md`
- 标题面向业务读者，避免纯技术缩写
