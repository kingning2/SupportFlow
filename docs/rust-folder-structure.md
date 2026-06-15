# Rust 文件夹结构

## 目标

Rust 拥有桌面应用编排、状态存储、配置、IPC、AI 工具链和跨端共享业务逻辑。全部实现在 `src-tauri/src/` 单 crate 内，按模块划分职责。

## 核心目录

- `src-tauri/src/cmd/`
  - Tauri command 入口（`#[cfg(feature = "desktop")]`）
- `src-tauri/src/context/`
  - 跨 Webview 共享状态、sidecar 运行态、运行时编排
- `src-tauri/src/services/`
  - Agent、Bridge、渠道等领域服务
- `src-tauri/src/events/`
  - 事件名、事件发射、事件载荷（desktop）
- `src-tauri/src/utils/`
  - 无全局 Store 的可复用逻辑
- `src-tauri/src/lib.rs`
  - 模块声明、Tauri 应用入口与 `.manage(...)`（desktop）

## 基础设施与共享模块

| 路径                   | 职责                                                            |
| ---------------------- | --------------------------------------------------------------- |
| `src/config/`          | `config.json` 字段、`Provider` 目录、`Context`/`Reply` IPC 契约 |
| `src/io/`              | 带 `[fs]` 审计日志的文件读写（对外别名 `crate::fs_io`）         |
| `src/channel_runtime/` | 群聊/单聊前缀、关键词、回复装饰（纯函数，无 Store）             |
| `src/process_runtime/` | `CommandSpec`、一次性子进程、`StdioJsonRpcRuntime`、进程 slot   |
| `src/cli/`             | `sf` 子命令（`commands/`、`paths/`、`runtime/`）                |
| `src/bin/sf.rs`        | CLI 二进制入口                                                  |

## `src/services/` 业务模块

```
services/
  agent/          # 工具链（read/write/bash/MCP/…）、rig LLM 编排、知识库、记忆、技能
  bridge/         # BridgeRuntime、AgentBridge、配置同步
  channel/        # 渠道配置等领域服务（desktop）
```

## `src/context/` 编排模块

```
context/
  agent_runtime/  # Agent 运行时门面（sidecar 延迟启动、控制台、会话、工作区）
  channel/        # 渠道 sidecar 协调、收件箱、状态、控制台 API
  process_hub.rs  # 构建 ProcessSharedContext（工作区 env 等）
  license_store.rs
```

`context/channel/config.rs` 负责 connect/disconnect 编排；勿与 `channel_runtime` 模块混淆。

## `src/python/` — Python 互操作层

**定位**：Rust 与仓库根目录 `channel_agent/` 的**唯一**对接层。  
spawn 子进程、解析脚本路径、stdio RPC 都放这里；`services/`、`context/` 只**调用** `crate::python::*`，不自己 `Command::new("python")`。

### 目录树

```
src/python/
  mod.rs              # 门面：模块声明 + 对外 re-export
  paths.rs            # 纯路径（无 AppHandle、无 Tauri）
  paths_desktop.rs    # 需 AppHandle / Resource 的路径解析（仅 desktop）
  markitdown.rs       # 一次性子进程：markitdown_convert.py → Markdown
  sidecar/            # 渠道 sidecar 长驻进程（wx / wework）
    mod.rs            # ChannelPythonSidecar、领域 RPC 方法
    spawn.rs          # 启动模式解析、spawn_sidecar 入口
    handler.rs        # Python → Rust 入站 RPC / 事件处理
    tauri_shell.rs    # Tauri externalBin sidecar 的 stdin/stdout 适配
```

进程读写、挂起 RPC、超时等底层能力在 `process_runtime/`；`python::sidecar` 只拼 `CommandSpec`、注册 handler、暴露领域方法。

### 模块职责

| 模块            | 依赖                                    | 职责                                                                               |
| --------------- | --------------------------------------- | ---------------------------------------------------------------------------------- |
| `paths`         | 无 Tauri                                | `channel_agent/` 根目录、sidecar exe、markitdown 脚本相对路径                      |
| `paths_desktop` | `AppHandle`                             | 从 Tauri Resource / 开发树解析运行时绝对路径                                       |
| `markitdown`    | `process_runtime`                       | 单次 `python <script> <file>`；读 `MARKITDOWN_SCRIPT`、`CHANNEL_MARKITDOWN_PYTHON` |
| `sidecar`       | `process_runtime`、`tauri-plugin-shell` | PyInstaller sidecar 或 dev `python -m channel`；双向 NDJSON RPC                    |

### 与 `channel_agent/` 的对应关系

| Python 源码                                                          | Rust 调用方式                                  | 进程模型                     |
| -------------------------------------------------------------------- | ---------------------------------------------- | ---------------------------- |
| `channel_agent/scripts/markitdown_convert.py`                        | `python::markitdown::convert_file_to_markdown` | **单次**子进程               |
| `channel_agent/channel/`（`python -m channel`）                      | `python::sidecar`                              | **长驻** sidecar + stdio RPC |
| `channel_agent/requirements-markitdown.txt`                          | markitdown 子进程依赖                          | 与 sidecar 依赖分离          |
| `channel_agent/requirements-sidecar.txt` / `requirements-wework.txt` | sidecar 构建与运行                             | 与 markitdown 分离           |

**不使用 PyO3**：长驻 SDK 与单次脚本均通过独立 Python 进程通信，保证崩溃隔离与 `ntwork` 等 SDK 的线程模型不受 Rust 运行时影响。

### 编写规则

1. **新 Python 能力先归类**
   - 长驻、双向通信 → `sidecar/`
   - 单次脚本、stdout 结果 → 顶层新文件（如 `markitdown.rs`）
   - 仅路径拼接 → `paths.rs`；需打包资源路径 → `paths_desktop.rs`

2. **禁止在其它目录复制 spawn 逻辑**
   - `services/agent/knowledge/` 里只允许 `use crate::python::markitdown`
   - `context::agent_runtime` 设置 `MARKITDOWN_SCRIPT` 应委托 `paths_desktop::resolve_markitdown_script`

3. **`mod.rs` 只做门面**；单文件超过 500 行按 `sidecar/` 方式拆子模块。

4. **feature 边界**
   - `paths`、`markitdown`：无 `desktop` feature（CLI `sf` 知识库也要用）
   - `paths_desktop`、`sidecar`：`#[cfg(feature = "desktop")]`

5. **环境变量约定**

   | 变量                        | 用途                                         |
   | --------------------------- | -------------------------------------------- |
   | `MARKITDOWN_SCRIPT`         | runtime 设置；覆盖 markitdown 脚本路径       |
   | `CHANNEL_MARKITDOWN_PYTHON` | markitdown 专用 Python                       |
   | `CHANNEL_PYTHON_EXECUTABLE` | 通用 Python（markitdown / dev sidecar 回退） |
   | `CHANNEL_SIDECAR_EXE`       | 覆盖 sidecar exe（测试用）                   |
   | `DEV_CHANNEL`               | 传给 sidecar 进程                            |

6. **与 `utils/platform/python.rs` 的分工**
   - `utils/platform/python`：Windows 启动标志、`resolve_python_executable`
   - `src/python`：业务级「调哪个脚本、传什么 env、RPC 契约」

### 典型调用链（知识库上传）

```
cmd::agent_ipc::agent_upload_knowledge
  → context::AgentRuntime::upload_knowledge_files
  → services::agent::knowledge::ingest
  → crate::python::markitdown::convert_file_to_markdown
  → channel_agent/scripts/markitdown_convert.py
```

### 典型调用链（渠道 sidecar）

```
context::AgentRuntime::start_sidecar_deferred
  → crate::python::spawn_sidecar
  → python::sidecar（Tauri externalBin 或 dev python -m channel）
  → process_runtime::StdioJsonRpcRuntime
```

## Sidecar 产物

- 源码：`channel_agent/`
- 构建：`pnpm run build:channel-sidecar`（PyInstaller one-file）
- 产物：`src-tauri/binaries/channel-sidecar-{target}{.exe}`

## 结构原则

1. `cmd` 只做命令入口与参数/返回值定义。
2. `context` 只做共享状态、sidecar 运行时协调、状态同步。
3. `services` 放可复用业务能力；`context` 负责编排与 `.manage` 状态。
4. `utils` 只做无状态工具逻辑。
5. 任何原本属于 Python 的应用层逻辑，优先迁移到 `context`、`services` 或 `utils`。
6. 不再向已删除的 `src-tauri/crates/*` 目录新增代码。
