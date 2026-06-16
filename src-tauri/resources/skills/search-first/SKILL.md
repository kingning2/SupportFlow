---
name: search-first
version: 1.0.0
description: 编码与排障前先检索现有实现、配置与文档，避免重复造轮子。
---

# 先搜索再动手

SupportFlow 内部开发/运维的检索优先技能。

## 原则

1. 改代码前先 `read` / `ls` / `memory_search` 查现有实现
2. 配置类问题先查 `config.json` 与文档，不凭记忆猜测
3. 渠道问题先确认 sidecar 状态与 `DEV_CHANNEL` 等环境变量
4. 能复用仓库内模式时，优先扩展而非新建平行逻辑

## 输出

- 说明已查阅的位置（文件/目录）
- 基于证据给出结论或修改建议
