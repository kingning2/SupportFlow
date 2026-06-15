//! 语言无关的子进程基础设施：一次性命令、长驻 stdio NDJSON RPC、懒加载 slot、任务级上下文。
//!
//! - **共享 context**：[`ProcessSharedContext`] — 工作区路径与跨进程环境变量，由桌面 `context::ProcessHub` 构建。
//! - **任务 context**：[`ProcessTaskContext`] — 绑定到单个 IO/RPC 异步任务的进程名，用于日志。
//! - **实例 state**：[`ProcessLocalState`] — 单个子进程句柄、stdin、挂起 RPC（不跨 Webview 共享）。
//!
//! Python / Rust 可执行文件均通过 [`CommandSpec`] 描述；业务适配层（`python::sidecar`、`license_verifier`）只负责拼 spec 与 RPC 分发。

mod backend;
mod env;
mod io;
mod launch;
mod local;
mod oneshot;
mod runtime;
mod shared;
mod slot;
mod spec;
mod stdin;
mod task;

pub use backend::ProcessBackend;
pub use env::piped_stdio;
pub use io::{forward_log_line, read_line_lossy};
pub use launch::LaunchMode;
pub use local::ProcessLocalState;
pub use oneshot::{run_async, run_sync, OneshotOutput};
pub use runtime::{InboundRpcHandler, StdioJsonRpcConfig, StdioJsonRpcRuntime};
pub use shared::ProcessSharedContext;
pub use slot::ProcessSlot;
pub use spec::{binary_in_dir, resolve_exe_from_env, CommandSpec};
pub use stdin::StdinLineWriter;
pub use task::ProcessTaskContext;
