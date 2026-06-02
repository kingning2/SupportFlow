# Logging and Errors

## Rule

- 业务调用 Tauri **Command** 走 `invokeWrapper(TauriCmd.…)`（`packages/shared/src/tauri-bridge/cmd/invoke.ts`）；失败会自动 `log` 并抛出 `InvokeError`。
- 写 Rust 日志走 **Event**：`tauri-bridge/cmd/log.ts` → `tauriEmit(TauriEvent.FeLog | TauriEvent.FeLogReq, …)`，级别用 `FeLogLevel` 枚举；由 `events/handlers/fe_log.rs` 写入 tracing（避免与 `invokeWrapper` 循环依赖）。
