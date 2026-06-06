# Rust 文件夹结构

## 目标

Rust 拥有桌面应用编排、状态存储、配置、IPC、AI 工具链和跨端共享业务逻辑。

## 核心目录

- `src-tauri/src/cmd/`
  - Tauri command 入口
- `src-tauri/src/context/`
  - 跨 Webview 共享状态、sidecar 运行态、运行时编排
- `src-tauri/src/events/`
  - 事件名、事件发射、事件载荷
- `src-tauri/src/utils/`
  - 无全局 Store 的可复用逻辑
- `src-tauri/src/lib.rs`
  - Tauri 应用入口与 `.manage(...)`

## Rust crate 分层

**原则：`crates/` 只放基础设施与可复用库；桌面业务编排放在 `src/`。**

- `src-tauri/crates/fs_io/`
  - 全仓库唯一的带日志文件 IO（禁止再复制 `fs.rs`）
- `src-tauri/crates/channel_runtime/`
  - 渠道消息规则（纯函数，无 Store）
- `src-tauri/crates/models/`
  - LLM Provider 协议与 `config.json` 模型（非桌面编排）
- `src-tauri/src/services/agent/`
  - Agent 工具引擎（原 `crates/agent`）
- `src-tauri/src/services/bridge/`
  - Bot 路由、`AgentBridge`（原 `crates/bridge`）
- `src-tauri/crates/cli/`
  - `sf` CLI 二进制

## `src/` 业务模块

```
src/
  services/
    agent/          # 工具链（read/write/bash/MCP/…）
    bridge/         # BridgeRuntime、AgentBridge
  context/
    channel/        # 渠道 sidecar、收件箱、配置（原 channel_*.rs）
      config.rs     # connect/disconnect（勿与 channel_runtime crate 混淆）
    agent_runtime.rs
  python/           # 全部 Python 子进程 / 脚本互操作（见下文）
  cmd/
    agent_ipc.rs    # 原 cmd/agent.rs（避免与 crate::agent 重名）
```

CLI（`sf`）通过 `tauri-app` 的 `default-features = false` 依赖 `services::*`，不链接 Tauri。

## `src/python/` — Python 互操作层

**定位**：Rust 与 `channel_agent/` 下 Python 代码的**唯一**对接层。  
spawn 子进程、解析脚本路径、stdio RPC 都放这里；`services/agent`、`context` 只**调用** `crate::python::*`，不自己 `Command::new("python")`。

### 目录树

```
src/python/
  mod.rs              # 门面：模块声明 + 对外 re-export（不写业务逻辑）
  paths.rs            # 纯路径（无 AppHandle、无 Tauri）
  paths_desktop.rs    # 需 AppHandle / Resource 的路径解析（仅 desktop）
  markitdown.rs       # 一次性子进程：markitdown_convert.py → Markdown
  client.rs           # 对 ChannelPythonSidecar 的薄 RPC 封装（仅 desktop）
  sidecar/            # 渠道 sidecar 长驻进程（wx / wework）
    mod.rs            # ChannelPythonSidecar、Rust↔Python RPC 分发
    spawn.rs          # 启动模式解析、spawn_sidecar 入口
    process.rs        # 进程生命周期、stdin/stdout 事件循环
```

### 模块职责

| 模块            | 依赖                 | 职责                                                                                    |
| --------------- | -------------------- | --------------------------------------------------------------------------------------- |
| `paths`         | 无 Tauri             | `channel_agent/` 根目录、sidecar exe、markitdown 脚本**相对路径**、`externalBin` 产物名 |
| `paths_desktop` | `AppHandle`          | 从 Tauri Resource / 开发树解析**运行时绝对路径**（如 `resolve_markitdown_script`）      |
| `markitdown`    | `std::process`       | `python <script> <file>` 单次转换；读 `MARKITDOWN_SCRIPT`、`CHANNEL_MARKITDOWN_PYTHON`  |
| `client`        | `sidecar`            | 给 `context::channel` 用的 `channel_start/stop/restart` 薄封装                          |
| `sidecar`       | `tauri-plugin-shell` | PyInstaller sidecar 或 dev `python -m channel`；双向 NDJSON RPC                         |

### 与 `channel_agent/` 的对应关系

| Python 源码                                     | Rust 调用方式                                                      |
| ----------------------------------------------- | ------------------------------------------------------------------ |
| `channel_agent/scripts/markitdown_convert.py`   | `python::markitdown::convert_file_to_markdown`（**不是** sidecar） |
| `channel_agent/channel/`（`python -m channel`） | `python::sidecar`（长驻 stdio RPC）                                |
| `channel_agent/requirements-markitdown.txt`     | markitdown 子进程依赖，与 sidecar 依赖分离                         |

### 编写规则

1. **新 Python 能力先归类**
   - 长驻、双向通信 → `sidecar/`
   - 单次脚本、stdout 结果 → 顶层新文件（如 `markitdown.rs`）或 `python/<name>.rs`
   - 仅路径拼接 → `paths.rs`；需打包资源路径 → `paths_desktop.rs`

2. **禁止在其它目录复制 spawn 逻辑**
   - `services/agent/knowledge/` 里只允许 `use crate::python::markitdown`，不得再维护第二份 `convert_file_to_markdown`。
   - `context::agent_runtime` 启动时设置 `MARKITDOWN_SCRIPT` 应委托 `paths_desktop::resolve_markitdown_script`，不内联路径拼接。

3. **`mod.rs` 只做门面**
   - 子模块实现细节不 re-export 到 crate 根，除非 `context` / `cmd` / `services` 需要。
   - 单文件超过 500 行按 `sidecar/` 方式拆子模块。

4. **feature 边界**
   - `paths`、`markitdown`：无 `desktop` feature（CLI `sf` 知识库上传也要用）。
   - `paths_desktop`、`client`、`sidecar`：`#[cfg(feature = "desktop")]`。

5. **环境变量约定**（在 `paths` / 各模块文档注释中维护）

   | 变量                        | 用途                                         |
   | --------------------------- | -------------------------------------------- |
   | `MARKITDOWN_SCRIPT`         | 由 runtime 设置；覆盖 markitdown 脚本路径    |
   | `CHANNEL_MARKITDOWN_PYTHON` | markitdown 专用 Python                       |
   | `CHANNEL_PYTHON_EXECUTABLE` | 通用 Python（markitdown / dev sidecar 回退） |
   | `CHANNEL_SIDECAR_EXE`       | 覆盖 sidecar exe（测试用）                   |
   | `DEV_CHANNEL`               | 传给 sidecar 进程                            |

6. **与 `utils/platform/python.rs` 的分工**
   - `utils/platform/python`：Windows 启动标志、`resolve_python_executable`（平台相关）。
   - `src/python`：业务级「调哪个脚本、传什么 env、RPC 契约」。

### 典型调用链（知识库上传）

```
cmd::agent_ipc::agent_upload_knowledge
  → context::AgentRuntime::upload_knowledge_files
  → services::agent::knowledge::ingest::ingest_bytes
  → services::agent::knowledge::document_parser::parse_document_file
  → crate::python::markitdown::convert_file_to_markdown   ← 必须走 python 层
  → channel_agent/scripts/markitdown_convert.py
```

### 典型调用链（渠道 sidecar）

```
context::AgentRuntime::start_sidecar_deferred
  → crate::python::spawn_sidecar
  → python::sidecar（Tauri externalBin 或 dev python -m channel）
```

## 结构原则

1. `cmd` 只做命令入口与参数/返回值定义。
2. `context` 只做共享状态、sidecar 运行时协调、状态同步。
3. `utils` 只做无状态工具逻辑。
4. 任何原本属于 Python 的应用层逻辑，优先迁移到 `context`、`utils` 或独立 crate。
