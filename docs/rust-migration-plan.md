# Python → Rust 迁移计划（SupportFlow）

本文档基于当前两个代码库的实际实现对比：

- Rust 桌面项目：`D:/Desktop/tauri-template`
- Python 原项目：`D:/Desktop/SupportFlow Agent`

目标：明确「尚未迁移项」并给出可执行的分阶段迁移路径。

---

## 1. 当前结论（2026-06 更新）

### 已完成迁移（Agent / Models 核心）

| 模块               | Python                                | Rust                                      | 状态                           |
| ------------------ | ------------------------------------- | ----------------------------------------- | ------------------------------ |
| Agent 流式执行     | `protocol/agent_stream`               | `protocol/stream/*`                       | ✅                             |
| 文件工具           | read/write/edit/ls/bash               | 同名                                      | ✅                             |
| send / memory 工具 | `tools/memory/*`                      | SQLite + embedding + hybrid search        | ✅                             |
| MCP                | `tools/mcp/*`                         | `tools/mcp/*`                             | ✅                             |
| Skills / Prompt    | `prompt/*`, `skills/*`                | 同名                                      | ✅                             |
| env_config         | `tools/env_config`                    | `~/.supportflow/.env` set/get/list/delete | ✅                             |
| web_fetch          | `tools/web_fetch` + `document_parser` | 同名 + `knowledge/document_parser`        | ✅                             |
| web_search         | 四后端 bocha/qianfan/zhipu/linkai     | 同名                                      | ✅                             |
| browser            | Playwright 服务                       | chromiumoxide + 系统 Chrome/Edge          | ✅（实现不同，能力对齐）       |
| HTTP 代理          | `common/http_proxy`                   | `models/http_proxy` + 接入 LLM/工具 HTTP  | ✅                             |
| 文档解析           | `knowledge/document_parser`           | PDF/docx/xlsx/pptx/文本                   | ✅                             |
| Models HTTP        | `openai_http_client`                  | `OpenAiHttpClient::from_config`           | ✅                             |
| 控制台 IPC         | sessions/knowledge/graph/channels     | `workspace_console.rs` + `cmd/agent.rs`   | ✅                             |
| vision             | `tools/vision` + `call_vision`        | `tools/vision` + trait 默认实现           | ✅（多厂商经 bot fallback）    |
| bridge             | `bridge/*`                            | `crates/bridge` + `AgentRuntime`          | ✅（语音/调度/会话持久化待补） |

### 未完成迁移（仍需推进）

1. **渠道层**（channel）仍走 Python sidecar；**bridge 核心已 Rust 化**（`crates/bridge`）
2. **语音 / 翻译**（voice、translate）— bridge 内为占位错误，待 Rust voice 模块
3. **部分厂商 vision 原生格式**（Claude/Gemini 专用 multimodal，当前多走 OpenAI-compat）
4. **CLI** — `crates/supportflow-cli`（`sf`）已覆盖主命令；Skill Hub 全格式安装等待补
5. **知识图谱增强**（ingest / list / read / graph / upload 已对齐 Python `KnowledgeService`）

---

## 2. 未迁移清单（按模块）

### A. 仍缺或部分对齐

- `channel/*` 全通道（`agent_get_channel_catalog` 等可代理 Python SupportFlow Agent）
- 各厂商 **非 OpenAI-compat** 的 vision 请求体（Claude/Gemini 等可后续 override `call_vision`）
- Deep Dream / memory flush 定时任务（可选）

---

## 3. 迁移目标层级

- **M1（桌面可用）**：控制台视图真实读写，去 placeholder
- **M2（能力对齐）**：Rust Agent + Models 达 Python 核心 80%+（**工具链已基本达成**）
- **M3（生态替代）**：通道/CLI Rust 化或 sidecar 标准化

---

## 4. 分阶段计划

### Phase 0：基线冻结 ✅

- 能力映射与验收口径（本文档）

### Phase 1：控制台占位 API ✅

- `workspace/sessions/index.json`、knowledge 目录扫描、wiki 链接图谱、config 渠道列表

### Phase 2：Agent 工具与 Memory ✅（核心已完成）

- SQLite memory、embedding、web_search、env_config、web_fetch、browser、document_parser、http_proxy

### Phase 3：Models 差距收敛（进行中）

- ✅ HTTP proxy、✅ `call_vision`（OpenAI-compat 默认）
- ⏳ 厂商专用 vision 体、native vendors 完整路径

### Phase 4：渠道与桥接（进行中）

- ✅ `crates/bridge`：`Bridge` / `AgentBridge` / `AgentInitializer` / `AgentEventHandler`
- ✅ Sidecar `agent.reply` → 完整 `Reply`（含 FILE / IMAGE_URL）
- ✅ 会话 SQLite 持久化（`conversation_store`，与 memory 共库）
- ⏳ 渠道仍 Python sidecar（`channels.status` / `channels.autostart` RPC）；调度器、Deep Dream 日终 flush

---

## 5. 优先级（建议开工顺序）

| 优先级 | 项                                                     | 状态      |
| ------ | ------------------------------------------------------ | --------- |
| P0     | 控制台 sessions/knowledge/channels API                 | ✅        |
| P0     | memory SQLite + 检索                                   | ✅        |
| P1     | proxy + 工具 HTTP 对齐                                 | ✅        |
| P1     | web_search / env_config / web_fetch / browser / vision | ✅        |
| P2     | 多通道、语音/翻译                                      | 待做      |
| P2     | CLI（`supportflow-cli`）                               | ✅ 主命令 |

---

## 6. 风险与应对

- **协议漂移**：golden fixtures 对比 Python/Rust tool 输出
- **双栈维护**：以 Rust 为主实现，Python SupportFlow Agent 仅作参考
- **browser 实现差异**：静态页用 `web_fetch`，JS 重页用 `browser`

---

## 7. 里程碑

- W1：Phase 1（控制台去占位）
- W2–W3：Phase 3 vision + models 细项
- W4+：Phase 4 渠道策略

---

## 8. Done 定义

- P0/P1 能力在 Rust 侧可用且无 Python 回退
- IPC 契约稳定、关键路径有测试与日志
