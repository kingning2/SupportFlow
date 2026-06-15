# Rust 编写规则

## 适用范围

- `src-tauri/src/**/*.rs`

## 角色定位

Rust 是本仓库的主业务层，负责桌面应用编排、状态、配置、IPC、AI 工具链与跨端共享逻辑。

## 分层规则

### `cmd/`

1. 只做 `#[tauri::command]` 入口。
2. 只定义参数、返回值、权限边界。
3. 具体逻辑一行委托给 `context`、`services` 或 `utils`。
4. 不在 `cmd` 中写业务流程、目录遍历、HTTP、文件下载、注册表解析。

### `context/`

1. 只放跨 Webview 共享状态与运行时编排。
2. 由 `lib.rs` 中 `.manage(...)` 持有。
3. 负责快照、状态更新、事件广播、sidecar 生命周期协调。
4. 不在 `context` 中写与平台强绑定的底层细节（下沉到 `utils` 或 `process_runtime`）。

### `services/`

1. 放 Agent、Bridge、渠道等领域服务与可复用业务逻辑。
2. 可被 `context`、`cmd`、`cli` 调用。
3. 不直接持有 Tauri `AppHandle` 的全局单例；需要时由 `context` 注入或传参。
4. 不在 `services` 里散落 `Command::new("python")`；Python 互操作走 `crate::python`。

### `utils/`

1. 只放无全局 Store 的业务逻辑与工具逻辑。
2. 可被 `cmd`、`context`、`services` 复用。
3. 不在 `utils` 里存跨 Webview 共享状态。

### 基础设施模块（`io`、`config`、`channel_runtime`、`process_runtime`）

1. **无 Tauri、无 Store** 的纯函数与类型放 `channel_runtime`、`config`（契约部分）、`io`。
2. 子进程与 stdio RPC 放 `process_runtime`；业务适配（sidecar、license verifier）只拼 spec 与 RPC 分发。
3. **禁止**在多个模块复制同一工具；文件 IO 统一 `crate::io`（`crate::fs_io`）。

## 测试

1. **禁止**在业务源码中使用 `#[cfg(test)] mod tests { ... }`。
2. 单元/集成测试统一放在 `src-tauri/tests/*.rs`（按领域分子目录亦可）。
3. 测试只通过 `pub` API 访问；仅供测试的辅助函数用 `#[doc(hidden)] pub fn` 标注。

## 文件体量

1. **单文件不超过 500 行**。超过时必须按职责拆成子模块。
2. 拆分原则：`cmd/` 按 IPC 领域；`context/` 按运行时职责；`python/` 按进程/RPC；`services/agent` 按工具或协议子域。
3. 多个 `impl Struct` 块若分布在子模块，跨模块调用的方法须标 `pub(crate)`。

## 代码规范

1. 能下沉到纯函数的逻辑，优先写成纯函数。
2. 能抽成独立类型的契约，不直接拼裸 JSON。
3. 能迁移出 Python 的应用层逻辑，优先迁移到 Rust。
4. 迁移完成后应删除 Python 旧分支，而不是长期双写。
5. 新增状态时，优先考虑是否应归属 `context`。
6. 条件分支优先采用错误前置、提前返回（guard clause）。

## 注释要求

1. 修改 `cmd/`、`context/`、`services/`、`utils/` 下函数时，必须写中文文档注释。
2. 文档注释至少说明：功能、参数、返回值。

## Python 互操作（`src/python/`）

1. 所有 Python 子进程与 `channel_agent/` 脚本调用集中在 `src/python/`（结构见 `rust-folder-structure.md`）。
2. `services/`、`context/` 禁止直接 `Command::new("python")`；通过 `crate::python::markitdown`、`crate::python::sidecar` 等公开 API 调用。
3. MarkItDown 与渠道 sidecar 是**两套进程模型**，不得合并进同一 Python 入口。
4. **不使用 PyO3**；需要 Python 能力时优先 subprocess / sidecar，而非嵌入解释器。
5. 新增 `python/*.rs` 时须在模块头用中文注释说明：对应哪份 Python 脚本、是否长驻、谁负责设置环境变量。

## IPC 规则

1. Command 走统一 Tauri `invoke`。
2. Event 走统一事件定义与共享载荷。
3. 改 IPC 时，要同步维护前端桥接与共享类型。

## 错误处理

1. 对外统一返回 `Result<_, String>` 时，错误文本要可诊断。
2. 不要把底层错误悄悄吞掉。
3. 日志要反映真实失败点，不写模糊描述。
