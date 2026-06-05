# Rust 编写规则

## 适用范围

- `src-tauri/src/**/*.rs`
- `src-tauri/crates/**/*.rs`

## 角色定位

Rust 是本仓库的主业务层，负责桌面应用编排、状态、配置、IPC、AI 工具链与跨端共享逻辑。

## 分层规则

### `cmd/`

1. 只做 `#[tauri::command]` 入口。
2. 只定义参数、返回值、权限边界。
3. 具体逻辑一行委托给 `context`、`utils` 或 crate。
4. 不在 `cmd` 中写业务流程、目录遍历、HTTP、文件下载、注册表解析。

### `context/`

1. 只放跨 Webview 共享状态与运行时编排。
2. 由 `lib.rs` 中 `.manage(...)` 持有。
3. 负责快照、状态更新、事件广播。
4. 不在 `context` 中写与平台强绑定的底层细节。

### `utils/`

1. 只放无全局 Store 的业务逻辑与工具逻辑。
2. 可被 `cmd`、`context`、其他 crate 复用。
3. 不在 `utils` 里存跨 Webview 共享状态。

### `crates/*`

1. **仅**放基础设施、协议适配、可被 CLI 与桌面共享的库（如 `fs_io`、`channel_runtime`）。
2. **不要**在 crate 内写 Tauri Store、sidecar 编排、渠道收件箱等业务；这些放在 `src/context`。
3. **禁止**在多个 crate 复制同一工具模块；文件 IO 统一 `fs_io`。
4. crate 间接口保持清晰，避免互相泄漏 `AppHandle` 等桌面细节。

## 测试

1. **禁止**在业务源码中使用 `#[cfg(test)] mod tests { ... }`。
2. 单元/集成测试统一放在 crate 根目录 `tests/*.rs`（`models` 等子 crate 同理使用各自 `crates/<name>/tests/`）。
3. 测试只通过 `pub` API 访问；仅供测试的辅助函数用 `#[doc(hidden)] pub fn` 标注，勿为测试放宽正常 API 可见性。

## 文件体量

1. **单文件不超过 500 行**（不含空行与注释亦可酌情，以 `wc -l` / IDE 行数为准）。超过时必须按职责拆成子模块（`foo/mod.rs` + `foo/bar.rs`），禁止继续堆在同一文件。
2. 拆分原则：`cmd/` 按 IPC 领域；`context/` 按运行时职责；`python/` 按进程/RPC；`services/agent` 按工具或协议子域。
3. 多个 `impl Struct` 块若分布在子模块，跨模块调用的方法须标 `pub(crate)`。

## 代码规范

1. 能下沉到纯函数的逻辑，优先写成纯函数。
2. 能抽成独立类型的契约，不直接拼裸 JSON。
3. 能迁移出 Python 的应用层逻辑，优先迁移到 Rust。
4. 迁移完成后应删除 Python 旧分支，而不是长期双写。
5. 新增状态时，优先考虑是否应归属 `context`。

## 注释要求

1. 修改 `cmd/`、`context/`、`utils/` 下函数时，必须写中文文档注释。
2. 文档注释至少说明：
   - 功能
   - 参数
   - 返回值

## IPC 规则

1. Command 走统一 Tauri `invoke`。
2. Event 走统一事件定义与共享载荷。
3. 改 IPC 时，要同步维护前端桥接与共享类型。

## 错误处理

1. 对外统一返回 `Result<_, String>` 时，错误文本要可诊断。
2. 不要把底层错误悄悄吞掉。
3. 日志要反映真实失败点，不写模糊描述。
