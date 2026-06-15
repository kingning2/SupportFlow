# 项目结构重构任务清单

## 定位

这份文档不是新的架构宣言，而是当前仓库结构治理的执行 backlog。
它把“Python 更薄、Rust 更重、TS 更清晰”拆成可分派、可验收的目录级与文件级任务，供后续架构收口、迁移治理与 code review 使用。

本文档默认以以下文档为准绳，不重复基础编码规范：

- `docs/rust-architecture.md`
- `docs/rust-folder-structure.md`
- `docs/rust-coding-rules.md`
- `docs/python-architecture.md`
- `docs/python-folder-structure.md`
- `docs/python-coding-rules.md`
- `docs/ts-architecture.md`
- `docs/ts-folder-structure.md`
- `docs/ts-coding-rules.md`

## 全局执行原则

### 1. 本文档只记录重构任务

- 不重复解释已有分层理念。
- 不展开实现细节之外的背景论证。
- 每项任务都必须指向具体目录或文件，而不是停留在抽象建议。

### 2. 任务分三阶段推进

- `P0`：统一边界，先解决双中心、混层与高风险入口。
- `P1`：消化历史层，分批迁移仍滞留在旧层级中的能力。
- `P2`：自动化固化，把文档约束转成规则、测试与 CI 约束。

### 3. 每项任务必须包含固定要素

- `目标`
- `涉及目录/文件`
- `动作`
- `迁移去向`
- `完成标准`
- `前置依赖`

### 4. 判断迁移去向的统一规则

- 需要 Tauri command 入口的，放 `src-tauri/src/cmd/`
- 需要跨 Webview 状态、快照、广播的，放 `src-tauri/src/context/`
- 需要长期保留的业务能力、应用服务、运行链路的，放 `src-tauri/src/services/`
- 无全局 Store 的通用逻辑才放 `src-tauri/src/utils/`
- Python 只能保留 SDK 适配、最小 RPC、`markitdown` 相关最小骨架
- TS 只承载前端页面、状态消费与 Rust IPC 薄桥接，不承载后端策略

## 总表

| 编号 | 优先级 | 主题                           | 当前问题                                                                       | 目标状态                                                              | 主要影响目录                                                                                  | 预估依赖 |
| ---- | ------ | ------------------------------ | ------------------------------------------------------------------------------ | --------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | -------- |
| T01  | P0     | Rust 分层入口收口              | `cmd`、`context`、`services`、`utils` 的落点规则虽然有文档，但未形成一致执行面 | 所有新旧入口都能按固定心智模型归类                                    | `src-tauri/src/cmd`、`src-tauri/src/context`、`src-tauri/src/services`、`src-tauri/src/utils` | 无       |
| T02  | P0     | ~~`crates/agent` 历史层治理~~  | ~~与 `src/services/agent` 并存~~                                               | **已完成**：`crates/agent` 已删除，能力在 `src/services/agent`        | `src-tauri/src/services/agent`                                                                | T01      |
| T03  | P0     | ~~`crates/bridge` 历史层治理~~ | ~~与 `src/services/bridge` 并存~~                                              | **已完成**：`crates/bridge` 已删除，能力在 `src/services/bridge`      | `src-tauri/src/services/bridge`                                                               | T01      |
| T04  | P0     | Python sidecar 应用层收口      | `channel_agent` 仍有应用编排与历史通用层残留                                   | Python 只保留 SDK 适配、RPC 骨架、脚本入口                            | `channel_agent/channel`、`channel_agent/bridge`、`channel_agent/common`                       | T01      |
| T05  | P1     | Channel 运行态分层收口         | Channel 的状态、配置、控制逻辑分散                                             | 状态归 `context/channel`，纯逻辑归 `services` 或 `utils`              | `src-tauri/src/context/channel`、`src-tauri/src/utils/channel.rs`                             | T01、T04 |
| T06  | P1     | Agent Runtime 门面化           | `context/agent_runtime` 可能直接依赖过多内部细节                               | 由少量应用服务门面承接运行时调用                                      | `src-tauri/src/context/agent_runtime`、`src-tauri/src/services/agent`                         | T02      |
| T07  | P1     | `utils` 业务化治理             | 部分 `utils` 文件已接近领域服务                                                | 仅保留通用逻辑，领域逻辑迁入 `services`                               | `src-tauri/src/utils`                                                                         | T01、T02 |
| T08  | P1     | Python 入口统一                | Python 相关调用已经集中到 `src/python`，但需进一步守住单入口规则               | 所有 Python 互操作只经 `src-tauri/src/python/*`                       | `src-tauri/src/python`、`src-tauri/src/services/agent`                                        | T04      |
| T09  | P1     | TS 共享层职责收口              | `packages/shared`、`packages/ui`、`apps/*` 后续容易互相回流业务                | `shared` 只保留桥接与共享类型，`ui` 只保留组件壳，业务页面留在 `apps` | `packages/shared`、`packages/ui`、`apps/*`                                                    | 无       |
| T10  | P2     | 依赖规则自动化                 | 分层规则主要依赖人工遵守                                                       | 用依赖规则与 CI 阻断非法依赖                                          | `.dependency-cruiser.cjs`、`.github/workflows`                                                | T01-T09  |
| T11  | P2     | 结构测试与契约校验             | 文档与代码之间缺少结构性回归检查                                               | 增加命令、事件、桥接与 Python 入口结构测试                            | `src-tauri`、`packages/shared`                                                                | T10      |
| T12  | P2     | 新人导览与迁移看板             | 结构规则对熟悉项目的人清楚，对新同学仍陡峭                                     | 用导览文档和迁移看板降低认知成本                                      | `docs`、`plan`                                                                                | T01-T11  |

## 阶段附录

## 当前执行状态

### 已完成的 P0 动作

- 已将桌面主流程中的 `crate::agent::*` 引用收口到 `crate::services::agent::*`
- 已将桌面主流程中的 `crate::bridge::*` 引用收口到 `crate::services::bridge::*`
- **已删除** `src-tauri/crates/agent`、`crates/bridge`、`crates/models`、`crates/fs_io`、`crates/channel_runtime`、`crates/process_runtime`、`crates/cli`；对应能力迁入 `src/` 单 crate 模块（`services/`、`config/`、`io/`、`channel_runtime/`、`process_runtime/`、`cli/`）
- 已验证 `src-tauri` 主工程可通过 `cargo check -p tauri-app --features desktop,channel-wework`
- Python 互操作文档已明确：**Tauri sidecar + markitdown 子进程**，**不使用 PyO3**

### 仍待继续的 P0 动作

- 对 `channel_agent/channel/channel_manager.py` 做职责裁边，拆出「必须留在 Python」与「应迁往 Rust」的逻辑清单
- 对 `channel_agent/bridge/*` 与 `channel_agent/common/*` 建立逐文件迁移判断表

### P0 收口结论（历史 `crates/` — 已下线）

#### `src-tauri/crates/agent` / `crates/bridge` / `crates/models` 等

- **状态**：目录已删除（2025–2026 结构收口）
- **现行落点**：
  - Agent → `src/services/agent/`（含 `rig` LLM 编排）
  - Bridge → `src/services/bridge/`
  - 配置与 Provider 契约 → `src/config/`
  - 文件 IO → `src/io/`（`crate::fs_io`）
  - 渠道消息规则 → `src/channel_runtime/`
  - 子进程/RPC → `src/process_runtime/`
  - CLI → `src/cli/` + `src/bin/sf.rs`
- **约束**：不再恢复 `src-tauri/crates/*` 工作区成员；新代码按 `docs/rust-folder-structure.md` 落模块

#### `channel_agent/channel/channel_manager.py`

- 保留在 Python：
  - 渠道类实例化
  - sidecar 内线程持有与启动
  - sidecar 本地 singleton 清理
  - SDK `startup()` / `stop()` 胶水
- 应迁往 Rust：
  - 桌面策略判断
  - 重试 / 重启策略
  - 用户可见状态同步
  - 跨 Webview 或跨会话编排

#### `channel_agent/bridge/*`

| 对象         | P0 结论        | 后续方向                                              |
| ------------ | -------------- | ----------------------------------------------------- |
| `context.py` | 保留为兼容 DTO | 新逻辑进入 Rust `config::bridge` / `services::bridge` |
| `reply.py`   | 保留为兼容 DTO | 新逻辑进入 Rust `config::bridge` / `services::bridge` |

#### `channel_agent/common/*`

| 对象              | P0 结论                                       | 后续方向                        |
| ----------------- | --------------------------------------------- | ------------------------------- |
| `http_proxy.py`   | 临时保留，供 sidecar 本地 `requests`/SDK 适配 | 桌面级代理策略统一由 Rust 决定  |
| `log.py`          | 临时保留，sidecar 本地日志入口                | 后续只作为 sidecar 本地薄封装   |
| `tmp_dir.py`      | 仅在 sidecar 本地文件落点确有必要时保留       | 若可复用为桌面通用逻辑则迁 Rust |
| `time_check.py`   | 倾向迁 Rust 或删除                            | 不应继续承载业务策略            |
| `expired_dict.py` | 倾向迁 Rust 或删除                            | 不应继续作为 Python 通用应用层  |
| `singleton.py`    | 仅在 sidecar 本地包装器仍依赖时保留           | 否则逐步退场                    |
| `utils.py`        | 逐函数审计，优先迁 Rust                       | 禁止继续增长为杂项应用层        |

## P0 统一边界

### P0-A Rust 分层收口

#### 任务 P0-A1

- `任务编号`：P0-A1
- `优先级`：P0
- `对象`：`src-tauri/src/cmd/*`
- `问题`：`cmd` 是架构上最容易长胖的层，当前需要核验每个 command 是否仍是薄封装。
- `目标`：所有 command 只负责 `#[tauri::command]`、参数/返回值定义与一跳委托。
- `动作`：
  - 审核 `agent_ipc.rs`、`channel_inbox.rs`、`lang.rs`、`license.rs`、`log.rs`、`session.rs`、`wework_accounts.rs`、`window.rs`
  - 标记其中是否存在目录遍历、平台分支、业务流程、长链路控制、Python 启动等待
  - 为每个超边界 command 建立迁移项，指定迁往 `context` 或 `services`
- `迁移去向`：`src-tauri/src/context/*` 或 `src-tauri/src/services/*`
- `完成标准`：
  - `cmd` 层只保留入口和委托
  - command 与 TS `TauriCmd` 的映射保持不变或有明确同步变更
- `前置依赖`：无

#### 任务 P0-A2

- `任务编号`：P0-A2
- `优先级`：P0
- `对象`：`src-tauri/src/context/channel/*`、`src-tauri/src/context/agent_runtime/*`
- `问题`：运行态与共享状态层已经建立，但后续极易混入平台细节和领域处理。
- `目标`：把 `context` 锁定为共享状态、快照构造、广播同步、运行时协调层。
- `动作`：
  - 逐一审视 `context/channel/bridge.rs`、`catalog.rs`、`config.rs`、`console_api.rs`、`inbox.rs`、`status.rs`、`wework_accounts.rs`
  - 逐一审视 `context/agent_runtime/channel.rs`、`channel_events.rs`、`console.rs`、`logs.rs`、`session.rs`、`setup.rs`、`stream.rs`、`workspace.rs`、`workspace_data.rs`
  - 识别其中的纯业务逻辑、平台实现细节、文件系统/网络流程
  - 为超边界逻辑建立迁出任务
- `迁移去向`：优先迁往 `src-tauri/src/services/*`，纯通用逻辑迁往 `src-tauri/src/utils/*`
- `完成标准`：
  - `context` 中不存在注册表解析、Python 脚本路径拼装、下载与安装流程实现
  - `context` 方法能用“状态 + 广播 + 协调”解释其职责
- `前置依赖`：无

### P0-B `crates/` 历史层治理 — **已完成**

`src-tauri/crates/*` 已全部删除并迁入 `src/` 单 crate。下列任务仅作历史记录。

#### 任务 P0-B1（已完成）

- `对象`：~~`src-tauri/crates/agent`~~ → `src-tauri/src/services/agent`
- `完成标准`：✅ 已达成

#### 任务 P0-B2（已完成）

- `对象`：~~`src-tauri/crates/bridge`~~ → `src-tauri/src/services/bridge`
- `完成标准`：✅ 已达成

### P0-C Python sidecar 瘦身

#### 任务 P0-C1

- `任务编号`：P0-C1
- `优先级`：P0
- `对象`：`channel_agent/channel/channel_manager.py`
- `问题`：这是 Python 侧最像应用编排层的文件，容易承载渠道生命周期、策略判断和运行态控制。
- `目标`：拆清“必须留在 Python 的 SDK 控制”和“应回到 Rust 的应用编排”。
- `动作`：
  - 识别文件中的生命周期控制、渠道选择、错误恢复、状态判断、配置解释
  - 把逻辑标记为三类：`SDK 必需` / `最小 RPC 必需` / `应迁往 Rust`
  - 为第三类建立对应 Rust 接收点，优先指向 `context/channel` 或 `services/bridge`
- `迁移去向`：`src-tauri/src/context/channel/*`、`src-tauri/src/services/bridge/*`
- `完成标准`：
  - `channel_manager.py` 只剩 sidecar 内必要的渠道分发与 SDK 控制
  - 不再持有桌面应用层策略
- `前置依赖`：P0-A2、P0-B2

#### 任务 P0-C2

- `任务编号`：P0-C2
- `优先级`：P0
- `对象`：`channel_agent/bridge/context.py`、`channel_agent/bridge/reply.py`
- `问题`：这两个文件属于历史 bridge 语义，需要确认是纯数据模型兼容层，还是仍携带应用逻辑。
- `目标`：如果仅是兼容 DTO，则保留最小形态；否则迁回 Rust。
- `动作`：
  - 审查 `Context` 与 `Reply` 相关结构是否仍被 Rust 侧或 sidecar 运行所必需
  - 若只是旧协议兼容，考虑在 Rust `config::bridge` / `services::bridge` 中建立唯一语义模型
  - 清点其调用方，决定是否进入退场列表
- `迁移去向`：`src-tauri/src/config/bridge/*` 或 `src-tauri/src/services/bridge/*`
- `完成标准`：
  - `channel_agent/bridge/*` 的存在理由清晰
  - 不再作为 Python 应用层的扩展入口
- `前置依赖`：P0-B2

#### 任务 P0-C3

- `任务编号`：P0-C3
- `优先级`：P0
- `对象`：`channel_agent/common/*`
- `问题`：`common` 是典型的历史“通用层”，容易残留非 SDK 必需能力。
- `目标`：缩减为 Python 必要工具集，其他逻辑回收至 Rust。
- `动作`：
  - 审核 `expired_dict.py`、`http_proxy.py`、`log.py`、`singleton.py`、`time_check.py`、`tmp_dir.py`、`utils.py`
  - 区分 `SDK 适配必需`、`sidecar 启动必需`、`可迁往 Rust`
  - 尤其标记与日志、代理、时间策略、临时目录、通用工具相关的重复能力
- `迁移去向`：`src-tauri/src/utils/*`、`src-tauri/src/services/agent/utils/*`
- `完成标准`：
  - `common` 不再扮演 Python 的“通用应用服务层”
  - 非 SDK 刚需代码有 Rust 替代路径
- `前置依赖`：P0-C1

## P1 消化历史层

### P1-A Channel 运行态与 Python 入口

#### 任务 P1-A1

- `任务编号`：P1-A1
- `优先级`：P1
- `对象`：`src-tauri/src/utils/channel.rs`
- `问题`：当前 `channel.rs` 的定位需进一步核定，防止在 `context/channel` 之外再长出第二套领域逻辑。
- `目标`：只保留无状态聚合、映射、解析等通用逻辑。
- `动作`：
  - 检查其中是否包含配置持久化、运行态判断、跨 Webview 同步、sidecar 生命周期控制
  - 如有，则拆向 `context/channel` 或新增 `services/channel`
- `迁移去向`：`src-tauri/src/context/channel/*` 或 `src-tauri/src/services/channel/*`
- `完成标准`：
  - `utils/channel.rs` 中的每个函数都能被解释为无状态复用逻辑
- `前置依赖`：P0-A2

#### 任务 P1-A2

- `任务编号`：P1-A2
- `优先级`：P1
- `对象`：`src-tauri/src/python/mod.rs`、`markitdown.rs`、`paths.rs`、`paths_desktop.rs`、`sidecar/mod.rs`、`sidecar/spawn.rs`、`sidecar/handler.rs`
- `问题`：Python 入口已经集中，但需要进一步固化“唯一入口层”规则。
- `目标`：Rust 中所有 Python 互操作都只经 `src/python/*` 发生。
- `动作`：
  - 审核公开 API 是否足以覆盖当前调用方
  - 清理 `services`、`context` 内部潜在的 Python 路径拼装、环境变量拼装、直接进程调用
  - 明确 `paths`、`paths_desktop`、`markitdown`、`sidecar` 的稳定边界
- `迁移去向`：保留在 `src-tauri/src/python/*`
- `完成标准`：
  - `services/*` 和 `context/*` 不直接承担 Python 启动细节
  - 文档能用一页说明 Rust 如何进入 Python
- `前置依赖`：P0-C1、P0-C3

### P1-B Agent Runtime 门面化

#### 任务 P1-B1

- `任务编号`：P1-B1
- `优先级`：P1
- `对象`：`src-tauri/src/context/agent_runtime/*`
- `问题`：运行时模块数较多，容易直接依赖 `services/agent` 的内部实现细节。
- `目标`：把 `agent_runtime` 压缩成运行时协调层，由少量服务门面承接能力。
- `动作`：
  - 清点 `channel.rs`、`console.rs`、`logs.rs`、`session.rs`、`setup.rs`、`stream.rs`、`workspace.rs`、`workspace_data.rs` 的调用面
  - 提取候选门面：`knowledge_service`、`session_service`、`provider_service`、`tool_service`、`workspace_service`
  - 明确运行时只依赖门面，不穿透到多级子模块
- `迁移去向`：新增或收口到 `src-tauri/src/services/agent/*`
- `完成标准`：
  - `context/agent_runtime` 面向少量稳定服务接口
  - 新增 agent 能力默认先进 `services` 门面而非运行时层
- `前置依赖`：P0-B1

#### 任务 P1-B2

- `任务编号`：P1-B2
- `优先级`：P1
- `对象`：`src-tauri/src/services/agent/*`
- `问题`：`services/agent` 规模已很大，容易继续横向膨胀。
- `目标`：形成按能力聚合的清晰模块，而不是按历史迁移来源堆积。
- `动作`：
  - 以 `knowledge`、`memory`、`prompt`、`protocol`、`skills`、`tools`、`utils` 为维度复核现有结构
  - 标记仍和旧 `crates/agent` 双写或高耦合的模块
  - 识别适合抽门面的入口文件
- `迁移去向`：仍保留在 `src-tauri/src/services/agent/*`
- `完成标准`：
  - `services/agent` 的对外入口清晰
  - 运行时层、命令层不再直接跨多层依赖内部模块
- `前置依赖`：P0-B1、P1-B1

### P1-C `utils` 业务化治理

#### 任务 P1-C1

- `任务编号`：P1-C1
- `优先级`：P1
- `对象`：`src-tauri/src/utils/skills_installer.rs`
- `问题`：文件名已带明显领域语义，可能并非纯工具层职责。
- `目标`：判断其是否应升级为领域服务。
- `动作`：
  - 审核其是否包含安装流程编排、外部依赖协调、业务策略或状态控制
  - 若超出纯工具，迁往 `services/agent/skills/*` 或独立 `services/skills/*`
- `迁移去向`：`src-tauri/src/services/agent/skills/*` 或 `src-tauri/src/services/skills/*`
- `完成标准`：
  - `utils/skills_installer.rs` 要么被证明是纯工具，要么完成迁移计划
- `前置依赖`：P1-B2

#### 任务 P1-C2

- `任务编号`：P1-C2
- `优先级`：P1
- `对象`：`src-tauri/src/utils/skills_config.rs`
- `问题`：配置解释逻辑若带策略含义，也容易演变成领域层。
- `目标`：把配置读取与业务决策拆开。
- `动作`：
  - 标记“纯配置模型/路径读取”和“技能业务策略”两类逻辑
  - 前者保留在 `utils`，后者迁往 `services/agent/skills/*`
- `迁移去向`：`src-tauri/src/services/agent/skills/*`
- `完成标准`：
  - `utils/skills_config.rs` 中不再夹带技能业务决策
- `前置依赖`：P1-B2

## P2 自动化固化

### P2-A TS 共享层收口

#### 任务 P2-A1

- `任务编号`：P2-A1
- `优先级`：P2
- `对象`：`packages/shared/src/tauri-bridge/*`
- `问题`：这是前后端桥接中心，后续最容易长出页面业务与前端状态策略。
- `目标`：严格保留为 invoke/event 薄桥接、共享枚举与类型入口。
- `动作`：
  - 核查 `cmd/*`、`enums/*`、`cache.ts`、`tauri-event.ts`、`window/main-window.ts`
  - 标记其中是否存在页面特定逻辑、视图状态组装、前端业务策略
  - 如有，迁回 `apps/*` 或 `packages/shared/src/desktop-shell/*`
- `迁移去向`：`apps/*` 或 `packages/shared/src/desktop-shell/*`
- `完成标准`：
  - `tauri-bridge` 中的文件都能解释为“桥接层”
- `前置依赖`：P0-A1

#### 任务 P2-A2

- `任务编号`：P2-A2
- `优先级`：P2
- `对象`：`packages/shared/src/desktop-shell/*`、`packages/ui/src/app-shell/*`
- `问题`：两层都在提供桌面前端壳，若边界不清，容易互相吞噬职责。
- `目标`：
  - `desktop-shell` 负责跨 app 的运行时壳、状态与 provider
  - `app-shell` 负责 UI 外壳与展示层组合
- `动作`：
  - 审核 `desktop-shell/config/*`、`events/*`、`guards/*`、`providers/*`、`store/*`
  - 审核 `app-shell/app-shell-layout.tsx`、`desktop-app-layout.tsx`、`desktop-app-root.tsx`、`channel-bridge.ts`
  - 为跨层泄漏的逻辑建立回迁项
- `迁移去向`：`packages/shared/src/desktop-shell/*` 或 `packages/ui/src/app-shell/*`
- `完成标准`：
  - `desktop-shell` 不直接承载页面视觉壳
  - `app-shell` 不重写运行时状态与桥接规则
- `前置依赖`：P2-A1

#### 任务 P2-A3

- `任务编号`：P2-A3
- `优先级`：P2
- `对象`：`apps/full/src/*`、`apps/wechat/src/*`、`apps/wework/src/features/wework/*`
- `问题`：应用层目录是最合理的业务着陆点，但需要防止回流公共层。
- `目标`：页面级业务、品牌化布局、view model 只留在 `apps/*`
- `动作`：
  - 审核 `apps/full/src/app/*`、`features/full/*`
  - 审核 `apps/wechat/src/app/*`、`wechat-page.tsx`
  - 审核 `apps/wework/src/features/wework/accounts/*`、`hooks/*`、`inbox/*`、`layout/*`、`views/*`
  - 标记哪些通用能力误入 app，哪些页面业务误入 shared/ui
- `迁移去向`：
  - 通用 UI 迁往 `packages/ui`
  - 共享桥接和类型迁往 `packages/shared`
  - 页面业务保留在 `apps/*`
- `完成标准`：
  - `apps/*` 成为唯一页面业务着陆层
  - 公共层不再反向吸收页面细节
- `前置依赖`：P2-A1、P2-A2

### P2-B 自动化约束

#### 任务 P2-B1

- `任务编号`：P2-B1
- `优先级`：P2
- `对象`：`.dependency-cruiser.cjs`
- `问题`：当前已有依赖规则文件，但需要承载真实分层约束。
- `目标`：把关键架构边界转成机器规则。
- `动作`：
  - 增加对 `apps/*`、`packages/ui`、`packages/shared` 的依赖方向约束
  - 增加对 `src-tauri/src/cmd`、`context`、`services`、`utils`、`python` 的非法依赖约束
  - 显式阻断 `services/*`、`context/*` 直接启动 Python 进程
- `迁移去向`：不涉及迁移
- `完成标准`：
  - 非法依赖能在本地与 CI 被稳定识别
- `前置依赖`：T01-T09

#### 任务 P2-B2

- `任务编号`：P2-B2
- `优先级`：P2
- `对象`：`.github/workflows/*`
- `问题`：如果 CI 不执行结构检查，规则难以长期成立。
- `目标`：把结构检查接入主线 CI。
- `动作`：
  - 在现有工作流中加入依赖规则检查、结构测试或契约校验
  - 明确失败时的报错信息，便于定位是哪一层违规
- `迁移去向`：不涉及迁移
- `完成标准`：
  - PR 阶段即可发现结构性回归
- `前置依赖`：P2-B1

#### 任务 P2-B3

- `任务编号`：P2-B3
- `优先级`：P2
- `对象`：`src-tauri`、`packages/shared`
- `问题`：命令名、事件名、桥接契约和 Python 入口规则需要结构性回归保护。
- `目标`：建立轻量但稳定的结构测试。
- `动作`：
  - 校验 Rust `generate_handler!` 与 TS `TauriCmd` 枚举同步
  - 校验事件名与共享 payload 契约同步
  - 校验 Python 互操作只经 `src-tauri/src/python/*`
- `迁移去向`：不涉及迁移
- `完成标准`：
  - 重构后命令链、事件链、Python 入口链不因结构调整而悄然失效
- `前置依赖`：P2-B1、P2-B2

## 验收标准

### 结构验收

- 新需求默认优先落 Rust，而不是回流 Python
- `cmd` 只做命令入口，不做业务流程
- `context` 只做共享状态、快照、广播、运行时协调
- `services` 成为桌面应用业务能力主落点
- `utils` 不再继续吸收领域服务
- `src/python` 成为 Rust 对 Python 的唯一入口层
- `channel_agent` 只保留 SDK 适配、最小 RPC 与 `markitdown` 骨架
- `packages/shared` 仍然是共享类型与桥接层
- `packages/ui` 仍然是 UI 外壳与组件层
- `apps/*` 成为页面业务唯一落点

### 依赖验收

- `apps/*` 不直接依赖 Rust 源码目录
- `packages/ui` 不依赖 `apps/*`
- `packages/shared/tauri-bridge` 不承载页面业务
- `src-tauri/src/cmd/*` 不直接依赖平台实现和长链路流程
- `src-tauri/src/context/*` 不直接承担 Python 启动与脚本路径拼装细节
- `src-tauri/src/services/*` 不直接 `Command::new("python")`

### 回归验收

- Rust command 调用链在重构后保持行为一致
- Python sidecar 启动链与 `markitdown` 调用链保持可用
- TS invoke/event 桥接链保持可用
- 新同学能在 30 秒内判断代码该放 `cmd`、`context`、`services`、`utils`、`python`、`apps`、`shared` 还是 `ui`

## 实施建议

### 推荐顺序

1. P0-A：先把 Rust 分层边界解释清楚
2. P0-B：再消除 `crates` 与 `services` 双中心
3. P0-C：同步压缩 Python 应用层残留
4. P1：处理 `agent_runtime`、`utils` 与 Python 入口收口
5. P2：最后用依赖规则、CI 与结构测试固化

### 使用方式

- 这份清单适合作为长期治理 backlog，而不是一次性大重构脚本
- 每完成一项任务，应同步更新本文档中的状态或拆出实现任务单
- 若后续新增目录或大模块，先决定其属于哪一层，再允许合并代码
