# Product Roadmap Execution Tracker

基于代码审计（Agent / Memory / RAG / Tool / Skills / Channel / Sidecar / Data）拆分的可执行任务队列。

## 当前执行位置

| 字段        | 值                                                                                           |
| ----------- | -------------------------------------------------------------------------------------------- |
| **Current** | _（无）_ — T002 已完成，待开始 T003                                                          |
| **Next**    | [`pending/T003-workflow-executor-skeleton.md`](./pending/T003-workflow-executor-skeleton.md) |

> 开始任务时：将文件从 `pending/` 移到 `in-progress/`，并更新本表 **Current**。  
> 完成任务时：移到 `completed/`，把 **Next** 指向下一个 `pending/` 文件。

## 文件夹说明

| 目录                             | 含义     |
| -------------------------------- | -------- |
| [`pending/`](./pending/)         | 待处理   |
| [`in-progress/`](./in-progress/) | 正在进行 |
| [`completed/`](./completed/)     | 处理完成 |

## 执行队列（按顺序）

| #   | 任务文件                                  | 优先级 | 主题                     |
| --- | ----------------------------------------- | ------ | ------------------------ |
| 1   | T001-workflow-runtime-mvp-design.md       | P0     | Workflow 运行时 MVP 设计 |
| 2   | T002-workflow-state-persistence.md        | P0     | 工作流状态持久化         |
| 3   | T003-workflow-executor-skeleton.md        | P0     | 工作流执行器骨架         |
| 4   | T004-rag-rerank-layer.md                  | P0     | RAG Rerank 层            |
| 5   | T005-rag-retrieval-eval-baseline.md       | P1     | RAG 检索评测基线         |
| 6   | T006-channel-adapter-contract.md          | P0     | 统一 Channel 适配器契约  |
| 7   | T007-channel-decouple-wework-hardcode.md  | P1     | 去除企微硬编码           |
| 8   | T008-channel-second-adapter-spike.md      | P1     | 第二通道 Spike           |
| 9   | T009-memory-split-conversation-db.md      | P1     | 会话库与记忆库拆分       |
| 10  | T010-user-profile-minimal-store.md        | P1     | 用户画像最小存储         |
| 11  | T011-conversation-summary-pipeline.md     | P2     | 对话自动总结管线         |
| 12  | T012-sidecar-multislot-feasibility.md     | P2     | 多 Sidecar 可行性        |
| 13  | T013-skills-version-and-params.md         | P2     | Skills 版本与参数化      |
| 14  | T014-multi-agent-role-model.md            | P2     | 多 Agent 角色模型        |
| 15  | T015-production-observability-baseline.md | P2     | 生产可观测性基线         |

## 三大里程碑

```mermaid
flowchart LR
  M1[M1 Workflow MVP] --> M2[M2 RAG Quality]
  M2 --> M3[M3 Multi-Channel]
  M3 --> M4[M4 Memory and Profile]
  M4 --> M5[M5 Production Hardening]
```

| 里程碑 | 任务范围  | 目标                               |
| ------ | --------- | ---------------------------------- |
| **M1** | T001–T003 | 可持久化、可恢复的最小工作流运行时 |
| **M2** | T004–T005 | 检索质量可度量、可迭代             |
| **M3** | T006–T008 | 第二通道可接入，不绑死企微         |
| **M4** | T009–T011 | 会话/记忆/画像分层清晰             |
| **M5** | T012–T015 | 扩展性与上线治理                   |

## 参考文档

- 成熟度审计结论：见对话记录（2026-06-15）
- 结构治理 backlog：[`project-structure-refactor-backlog.md`](./project-structure-refactor-backlog.md)
- Sidecar IPC 草案：[`rust-sidecar-async-ipc-architecture.md`](./rust-sidecar-async-ipc-architecture.md)
