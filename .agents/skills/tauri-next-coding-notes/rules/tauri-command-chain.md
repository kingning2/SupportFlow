# Tauri Command Chain

新增或修改 **Tauri Command** 时，按顺序改齐以下位置（详见 [`docs/development-rules/fullstack-ipc.md`](../../../../docs/development-rules/fullstack-ipc.md)）：

1. Rust command 文件（`src-tauri/src/cmd/*.rs`）
2. Rust 模块导出（`src-tauri/src/cmd/mod.rs`）
3. Rust handler 注册（`src-tauri/src/lib.rs`）
4. 前端枚举（`packages/shared/src/tauri-bridge/enums/tauri-cmd.ts` 增加 `TauriCmd` 成员，字符串与 Rust 命令名一致）
5. 前端封装函数（`packages/shared/src/tauri-bridge/cmd/*.ts` 使用 `invokeWrapper(TauriCmd.Xxx, …)`）

前端**不要**使用裸字符串命令名（统一 `TauriCmd` + `invokeWrapper`）。
