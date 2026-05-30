# 后端（Rust / Tauri）编写规范

> 各职位对应的目录说明见 [roles-and-directories.md](./roles-and-directories.md)。

入口：`src-tauri/src/lib.rs`（`generate_handler!`、`.manage`、`events::setup`）。

## 分层（必须遵守）

```
cmd/          → #[tauri::command] 薄入口，一行委托
context/      → .manage 的跨 Webview 共享态（SessionStore 等）
utils/        → 无 Store 的可复用业务与通用逻辑
utils/platform/ → 按 OS 分文件实现，门面模块做 #[cfg] 转发
events/       → 事件名、emit、listen 注册
```

| 层           | 可以做                                            | 不可以做                              |
| ------------ | ------------------------------------------------- | ------------------------------------- |
| `cmd`        | 参数校验、调用 `context`/`utils`、记录 cmd 级日志 | 注册表遍历、HTTP 下载细节、长平台分支 |
| `context`    | Store 读写、快照、广播 `events`                   | 注册表/文件系统等平台细节             |
| `utils`      | 业务流程、调用 `context` 更新进度并广播           | 把 Store 定义放在 utils               |
| `platform/*` | 单平台实现                                        | 在共享文件堆大量 `#[cfg]`             |

新增 `pub fn` 需有文档注释，说明功能、参数与返回值（见 `.cursor/rules/tauri-rust-layering.mdc` 示例）。

## Command 约定

- 命令名与前端 `TauriCmd` 枚举字符串**完全一致**（蛇形命名，如 `get_app_session`）。
- 新增 command 后必须在 `lib.rs` 的 `generate_handler!` 中注册，并在 `cmd/mod.rs` 导出。
- 需要跨窗同步的状态变更后，通过 `events` 发射（如 `session_changed_all`），不要指望前端各自轮询。

## Event 约定

- 事件名字符串只在 `events/names.rs` 定义，与前端 `src/enums/tauri-event.ts` 同步。
- Rust → 前端：走 `events::emit` 辅助函数。
- 前端 → Rust：在 `events/handlers/` 注册 listen，**不要**为同类能力再开 invoke command（例如前端日志）。

## 类型与契约

- 与前端共享的 DTO 使用 `#[typeshare]`，修改后执行：

  ```bash
  bun run generate:contracts
  ```

- 生成物：`src/generated/contracts.ts`（勿手改）。
- 校验 CI 可用：`bun run check:contracts`。

## 错误与日志

- Command 对外常用 `Result<T, String>`；错误信息应对前端/日志可读，避免 `unwrap()` panic 到用户侧。
- 使用项目宏 `log_info!` / `log_error!` 等，保持与 tracing 配置一致。
- 在 `cmd` 成功路径可打简短结构化日志，便于桌面端排障。

## 平台代码

- 对外单一 API 放在 `utils/platform/foo.rs`。
- Windows / macOS 实现分别在 `utils/platform/windows/`、`utils/platform/macos/`。
- 共享 IPC 参数结构体可放在门面模块，解析逻辑进子模块。

## 提交前（Rust）

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
# 或
bun run check:rust
```

与前端一并检查时：`bun run check`（format + eslint + tsc）。
