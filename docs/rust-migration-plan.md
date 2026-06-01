# Python → Rust 迁移计划（SupportFlow）

本文档基于当前两个代码库的实际实现对比：

- Rust 桌面项目：`D:/Desktop/tauri-template`
- Python 原项目：`D:/Desktop/CowAgent`

目标：明确「尚未迁移项」并给出可执行的分阶段迁移路径。

---

## 1. 当前结论

### 已完成迁移（主干能力）

- Agent 核心流式执行（`run_stream`、tool loop、cancel、skills、MCP 基础链路）已在 Rust 落地。
- 多模型工厂与主流 OpenAI-compatible 路径已在 Rust 落地。
- Tauri 侧已具备完整 IPC 主链路（发送消息、流式事件、模型配置、技能刷新、日志流等）。

### 未完成迁移（需要继续）

仍有一批 Python 能力没有在 Rust 端形成等价实现，尤其是：

1. 渠道层（channel）
2. 知识库与图谱完整链路
3. 会话持久化与多会话管理
4. memory 的 SQLite/embedding 体系
5. 语音、翻译、视觉、浏览器工具等高级能力
6. 部分厂商原生 API、代理与 vision

---

## 2. 未迁移清单（按模块）

## A. IPC/控制台占位能力（Rust 端已声明但未实现）

文件：`src-tauri/src/cmd/agent.rs`

- `agent_list_sessions`：目前返回空数组
- `agent_list_knowledge`：目前返回空数组
- `agent_read_knowledge`：目前返回空内容
- `agent_get_knowledge_graph`：目前返回空图
- `agent_list_channels`：目前返回空数组

说明：这些是控制台管理页「最直观的未完成项」。

## B. Agent 深层能力（README 明确 next/TODO）

文件：`src-tauri/crates/agent/README.md`

- 完整 `agent/memory`（SQLite + embedding）未迁完
- 可选工具如 `web_search`、`env_config` 未在 Rust 侧形成完整对等

## C. Models 能力缺口（README 明确 TODO）

文件：`src-tauri/crates/models/README.md`

- `channel reply()` 流程未迁移
- 厂商原生 API（Baidu/Gemini/讯飞）未完成 Rust 原生等价
- HTTP 代理（`common/http_proxy`）未迁移
- `call_vision` 未迁移
- DeepSeek thinking 模式附加参数未完全对齐 Python

## D. Python 端仍是主实现的系统

参考目录：`D:/Desktop/CowAgent`

- `channel/`：web、微信、企微、飞书、钉钉、QQ 等多通道
- `bridge/`：消息桥接、事件处理、初始化
- `voice/`：多厂商 TTS/ASR 语音链路
- `translate/`：翻译能力
- `agent/tools/browser`、`agent/tools/vision`、`agent/tools/web_search`
- `agent/memory/*`：分块、摘要、embedding、存储管理等
- `cli/`：CLI 命令体系

---

## 3. 迁移目标定义

定义三个迁移目标层级，避免“一口气全迁”风险：

- **M1（桌面可用）**：控制台所有视图都能真实读写，不再 placeholder
- **M2（能力对齐）**：Rust Agent + Models 达到 Python 核心能力 80%+
- **M3（生态替代）**：可选实现 Rust 通道/CLI，Python 仅保留兼容层或退役

---

## 4. 分阶段迁移计划

## Phase 0：基线冻结（1-2 天）

目标：先锁定范围和验收口径，防止迁移中反复返工。

- 输出 Python→Rust 能力映射表（功能、输入输出、配置项、异常语义）
- 给每个未迁移项标注优先级：P0/P1/P2
- 约定统一验收方式（集成测试 + 控制台手工回归）

交付物：

- 本文档（已完成）
- 一份 `migration-mapping` 表（建议后续补充到 `docs/`）

## Phase 1：补齐控制台占位接口（3-5 天）

目标：把用户可见的占位能力补成可用最小闭环。

实施项：

- `agent_list_sessions`：会话索引读取（先文件索引，再演进 DB）
- `agent_list_knowledge` + `agent_read_knowledge`：读取 workspace 知识文档
- `agent_get_knowledge_graph`：先做最小图谱（文档节点 + 引用关系）
- `agent_list_channels`：先提供「配置态」列表（active/label），后续接运行态

验收标准：

- 控制台对应页面不再显示空占位
- 至少 1 条 e2e 流程可跑通：查询知识 -> 读取内容 -> 图谱展示

## Phase 2：Memory 与工具能力增强（1-2 周）

目标：把 Rust Agent 的“可用”提升到“可持续生产”。

实施项：

- memory SQLite 存储落地（含 schema、迁移脚本、版本兼容）
- embedding provider 抽象 + fallback
- 迁移 `web_search`、`env_config`、`vision`（按优先级分批）
- 明确工具超时、重试、错误码规范

验收标准：

- memory 检索命中率与 Python 基线在同一量级
- 工具调用失败可观测（日志 + 前端展示）

## Phase 3：Models 差距收敛（1-2 周）

目标：Rust models 与 Python models 行为对齐。

实施项：

- `call_vision` 能力补齐
- HTTP proxy 链路接入
- 补厂商原生 API 路径（按实际业务流量优先）
- DeepSeek thinking 参数语义与 Python 对齐

验收标准：

- 选定厂商回归测试全通过
- Rust 与 Python 在同请求下输出结构一致（允许文本差异，不允许协议差异）

## Phase 4：渠道与桥接迁移策略（2-4 周，可并行）

目标：决定并推进 channel/bridge 的 Rust 化路线。

建议两种路径二选一：

- 路径 A：Rust 原生通道框架（长期收益高，初期成本高）
- 路径 B：Python 通道保留为 sidecar，Rust 通过标准协议接入（短期落地快）

验收标准：

- 至少 1 个非 web 通道完成全链路打通
- 故障恢复、重连、消息去重具备最小保障

---

## 5. 优先级建议（建议直接开工顺序）

- **P0**：控制台 placeholder API（sessions/knowledge/channels）
- **P0**：memory 持久化基础（SQLite + 读取/检索稳定性）
- **P1**：models 的 proxy + vision + DeepSeek thinking 对齐
- **P1**：web_search/env_config/vision 工具补齐
- **P2**：多通道 Rust 原生化、CLI 完整迁移、语音/翻译生态迁移

---

## 6. 风险与应对

- 协议漂移风险：Python 与 Rust 输出结构不一致
  - 应对：统一契约测试（golden fixtures）

- 厂商行为差异风险：同 API 不同边界条件
  - 应对：每家厂商建立最小回归集（超时、限流、空响应、tool-call）

- 迁移期双栈维护成本高
  - 应对：明确“谁是主实现”，避免双向同时改

---

## 7. 里程碑建议（可按周排期）

- W1：Phase 0-1 完成（控制台去占位）
- W2-W3：Phase 2 完成（memory + 工具核心）
- W4：Phase 3 完成（models 对齐）
- W5+：Phase 4 逐步推进（渠道策略落地）

---

## 8. Done 定义（DoD）

判定“迁移完成”必须同时满足：

- 功能：Rust 侧覆盖既定范围内所有 P0/P1 能力
- 契约：IPC 与模型输出结构稳定，不依赖 Python 回退
- 质量：关键路径具备测试与日志可观测
- 运维：配置、升级、错误恢复流程文档化
