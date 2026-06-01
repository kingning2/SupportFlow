# Rust 迁移 Issue 拆分（可直接建单）

基于 `docs/rust-migration-plan.md` 拆分。  
建议在 GitHub 使用以下标签：

- `migration`
- `backend-rust`
- `frontend`
- `models`
- `agent`
- `p0` / `p1` / `p2`

---

## Milestone M1（桌面可用，P0）

## Issue 1：建立 Python→Rust 能力映射基线

- **优先级**：P0
- **建议标题**：`chore(migration): 建立 Python→Rust 能力映射表与验收口径`
- **目标**：冻结迁移范围，避免后续反复返工。
- **范围**：
  - 新建 `docs/migration-mapping.md`
  - 列出 Python 模块与 Rust 对应实现/缺口
  - 定义统一验收项（接口、行为、异常、日志）
- **验收标准**：
  - 映射表覆盖 `channel/agent/models/bridge/memory/tools`
  - 每条缺口标注 P0/P1/P2
  - 团队确认后不再随意扩 scope
- **依赖**：无

## Issue 2：实现 sessions 列表 IPC（替换 placeholder）

- **优先级**：P0
- **建议标题**：`feat(tauri): 实现 agent_list_sessions 会话索引读取`
- **目标**：控制台会话列表有真实数据来源。
- **范围**：
  - `src-tauri/src/cmd/agent.rs`：实现 `agent_list_sessions`
  - `src-tauri/src/context/agent_runtime.rs`：新增会话索引读取逻辑（先文件方案）
  - 前端会话面板消费真实数据
- **验收标准**：
  - 接口不再返回固定空数组
  - 前端能显示 >=1 条历史会话（含更新时间）
  - 提供基础测试（至少 1 个 Rust 单测）
- **依赖**：Issue 1

## Issue 3：实现 knowledge 列表与文档读取 IPC

- **优先级**：P0
- **建议标题**：`feat(tauri): 实现 agent_list_knowledge 与 agent_read_knowledge`
- **目标**：Knowledge 页面可读取 workspace 文档。
- **范围**：
  - `src-tauri/src/cmd/agent.rs`：实现 `agent_list_knowledge`、`agent_read_knowledge`
  - `src-tauri/src/context/agent_runtime.rs`：扫描/读取知识目录（限定 workspace 安全路径）
  - 前端 knowledge 页接入真实数据渲染
- **验收标准**：
  - 能列出文档并读取内容
  - 路径越权（`..`）被拒绝
  - 前端不再展示纯占位空态
- **依赖**：Issue 1

## Issue 4：实现最小知识图谱 IPC

- **优先级**：P0
- **建议标题**：`feat(tauri): 实现 agent_get_knowledge_graph 最小图谱`
- **目标**：先给出可用图谱 MVP（节点+边）。
- **范围**：
  - `src-tauri/src/cmd/agent.rs`：实现 `agent_get_knowledge_graph`
  - 图谱生成规则：文档节点 + 引用关系（或目录关系）
  - 前端图谱页可展示并容错空图
- **验收标准**：
  - 返回非固定空结构（有数据时含 nodes/links）
  - 无知识文档时返回空图但有稳定状态提示
- **依赖**：Issue 3

## Issue 5：实现 channels 配置态列表 IPC

- **优先级**：P0
- **建议标题**：`feat(tauri): 实现 agent_list_channels 配置态通道列表`
- **目标**：Channels 页展示可识别的通道状态（先配置态）。
- **范围**：
  - `src-tauri/src/cmd/agent.rs`：实现 `agent_list_channels`
  - 从配置读取通道开关与标签（active/label）
  - 前端 channels 页面改为真实列表
- **验收标准**：
  - 接口不再返回固定空数组
  - UI 展示通道名、启用状态
  - 兼容未知通道类型（不崩溃）
- **依赖**：Issue 1

## Issue 6：M1 回归与端到端验收

- **优先级**：P0
- **建议标题**：`test(migration): M1 控制台去占位回归与 e2e 验收`
- **目标**：确认 M1 功能对用户可用。
- **范围**：
  - 补 1 条 e2e 流程：知识列表 -> 读取内容 -> 图谱查看
  - 补 sessions/channels 基础回归
  - 更新 `docs/agent-console.md` 状态描述
- **验收标准**：
  - 核心流程可复现通过
  - 文档与代码状态一致
- **依赖**：Issue 2/3/4/5

---

## Milestone M2（能力对齐，P0/P1）

## Issue 7：落地 memory SQLite 存储层

- **优先级**：P0
- **建议标题**：`feat(agent): 落地 memory SQLite 存储与迁移脚本`
- **目标**：替换纯文件 fallback，建立可扩展 memory 基础层。
- **范围**：
  - `src-tauri/crates/agent` 新增 memory 存储抽象与 SQLite 实现
  - 增加 schema 初始化与版本迁移
  - 保留文件 fallback（故障降级）
- **验收标准**：
  - 首次启动自动建库
  - 升级路径可迁移旧数据
  - 存储故障可回退并记录日志
- **依赖**：Issue 1

## Issue 8：embedding provider 抽象与 fallback

- **优先级**：P0
- **建议标题**：`feat(agent): 新增 embedding provider 抽象与关键词 fallback`
- **目标**：memory_search 从“可用”走向“效果稳定”。
- **范围**：
  - provider trait + 至少一种默认实现
  - fallback 到关键词检索策略
  - 与 memory_search/memory_get 集成
- **验收标准**：
  - embedding 不可用时功能可降级
  - 检索结果结构稳定
- **依赖**：Issue 7

## Issue 9：迁移 web_search 工具到 Rust

- **优先级**：P1
- **建议标题**：`feat(agent): 迁移 web_search 工具并接入 tool_manager`
- **目标**：补齐高频检索工具能力。
- **范围**：
  - Rust 实现 `web_search` 工具
  - 注册到 tool_manager，支持配置开关
  - 日志与错误码对齐当前工具风格
- **验收标准**：
  - 工具可被 Agent 调用并返回结构化结果
  - 超时/失败能正确反馈前端
- **依赖**：Issue 1

## Issue 10：迁移 env_config 工具到 Rust

- **优先级**：P1
- **建议标题**：`feat(agent): 迁移 env_config 工具并补齐配置安全约束`
- **目标**：与 Python 侧 env 配置能力对等。
- **范围**：
  - Rust 实现 `env_config`
  - 限制可读写变量范围
  - 返回标准化执行结果
- **验收标准**：
  - 可读写白名单内变量
  - 非白名单操作被拒绝并有明确错误
- **依赖**：Issue 1

## Issue 11：迁移 vision 工具基础能力

- **优先级**：P1
- **建议标题**：`feat(agent): 迁移 vision 工具基础链路（输入校验 + 调用接口）`
- **目标**：为 models `call_vision` 对齐打前置基础。
- **范围**：
  - Rust 侧 vision tool MVP
  - 输入文件校验、大小限制、格式限制
  - 与模型层调用打通
- **验收标准**：
  - 成功路径可跑通
  - 非法文件输入能被拒绝
- **依赖**：Issue 15

## Issue 12：统一工具超时/重试/错误码规范

- **优先级**：P1
- **建议标题**：`refactor(agent): 统一工具超时重试与错误码语义`
- **目标**：提升工具执行可观测性与一致性。
- **范围**：
  - 定义统一错误码和重试策略
  - 统一日志字段（tool/request_id/elapsed/status）
  - 更新相关工具实现
- **验收标准**：
  - 同类错误返回语义一致
  - 前端可稳定展示失败原因
- **依赖**：Issue 9/10/11（可并行推进，后收敛）

---

## Milestone M2（Models 对齐，P1）

## Issue 13：接入 HTTP proxy 到 Rust models

- **优先级**：P1
- **建议标题**：`feat(models): 接入 HTTP proxy 配置与请求透传`
- **目标**：补齐 Python `common/http_proxy` 能力缺口。
- **范围**：
  - `src-tauri/crates/models` HTTP client 层接入 proxy
  - 配置项读取与安全兜底
  - 关键模型路径回归
- **验收标准**：
  - 开启 proxy 后请求可成功
  - proxy 不可达时错误可识别
- **依赖**：Issue 1

## Issue 14：对齐 DeepSeek thinking 参数语义

- **优先级**：P1
- **建议标题**：`feat(models): 对齐 DeepSeek thinking-mode 参数与行为`
- **目标**：与 Python 行为保持一致。
- **范围**：
  - 对照 Python deepseek 实现补齐请求参数
  - 对齐响应映射与异常分支
  - 增加对照测试
- **验收标准**：
  - 同配置下 Rust/Python 请求结构一致
  - 回归测试通过
- **依赖**：Issue 1

## Issue 15：实现 models call_vision 能力

- **优先级**：P1
- **建议标题**：`feat(models): 实现 call_vision 与多模态输入适配`
- **目标**：补齐 models README TODO。
- **范围**：
  - 增加 `call_vision` 接口实现
  - 支持图片输入与错误处理
  - 与 agent vision tool 打通
- **验收标准**：
  - vision 请求能正常返回
  - 不支持模型时有明确错误
- **依赖**：Issue 13（建议）

## Issue 16：补齐厂商原生 API 路径（分批）

- **优先级**：P1
- **建议标题**：`feat(models): 补齐厂商原生 API 路径（Baidu/Gemini/讯飞）`
- **目标**：减少仅兼容层带来的能力差距。
- **范围**：
  - 分 3 个子任务（Baidu、Gemini、讯飞）
  - 每家补最小可用路径与错误处理
  - 不影响既有 openai-compatible 路径
- **验收标准**：
  - 每家至少 1 条真实调用链路通过
  - 契约与日志满足统一规范
- **依赖**：Issue 13

## Issue 17：channel reply 流程迁移到 Rust models

- **优先级**：P1
- **建议标题**：`feat(models): 迁移 channel reply() 流程并统一输出结构`
- **目标**：为后续通道 Rust 化提供模型输出标准接口。
- **范围**：
  - 实现 reply 流程核心接口
  - 对齐 Python 回复结构
  - 增加契约测试
- **验收标准**：
  - reply 输出结构稳定
  - 与现有前端/桥接接口可对接
- **依赖**：Issue 16

---

## Milestone M3（生态替代，P2）

## Issue 18：确定渠道迁移路线（A 原生 / B sidecar）

- **优先级**：P2（策略先行）
- **建议标题**：`decision(migration): 确认 channel/bridge Rust 化路线（A/B）`
- **目标**：避免在架构分歧上消耗开发周期。
- **范围**：
  - 输出 ADR：A Rust 原生 vs B Python sidecar
  - 成本、风险、上线节奏对比
  - 结论与里程碑签字
- **验收标准**：
  - ADR 文档合并
  - 明确唯一执行路线
- **依赖**：Issue 17（建议先完成模型对齐）

## Issue 19：打通首个非 web 通道端到端

- **优先级**：P2
- **建议标题**：`feat(channel): 打通首个非 web 通道端到端（建议飞书或钉钉）`
- **目标**：验证渠道迁移路线可落地。
- **范围**：
  - 选择一个高价值通道（飞书/钉钉）
  - 完成收消息 -> 调 Agent -> 回消息闭环
  - 增加重连与去重最小机制
- **验收标准**：
  - 长连接稳定运行
  - 消息可收发，失败可恢复
- **依赖**：Issue 18

## Issue 20：CLI 迁移规划与最小命令落地

- **优先级**：P2
- **建议标题**：`feat(cli): 迁移 CLI 最小命令集并对齐文档`
- **目标**：为后续运维/自动化提供 Rust 入口。
- **范围**：
  - 先迁移最小命令集（如 process/skill/knowledge 选 1-2）
  - 输出命令契约与帮助文档
  - 与桌面配置共享基础设施
- **验收标准**：
  - 命令可执行且输出稳定
  - 文档可指导新成员使用
- **依赖**：Issue 18（可弱依赖）

---

## 里程碑打包建议

- **W1（M1）**：Issue 1-6
- **W2-W3（M2-agent）**：Issue 7-12
- **W4（M2-models）**：Issue 13-17
- **W5+（M3）**：Issue 18-20

---

## GitHub 建单模板（复制即用）

```md
## 背景

- 对应迁移计划：`docs/rust-migration-plan.md`
- 对应 issue 拆分：`docs/rust-migration-issues.md`

## 目标

- <一句话目标>

## 范围

- [ ] <任务1>
- [ ] <任务2>
- [ ] <任务3>

## 验收标准

- [ ] <验收1>
- [ ] <验收2>

## 依赖

- 前置：#<issue-id>
- 阻塞：#<issue-id>

## 风险

- <风险与回滚方案>
```
