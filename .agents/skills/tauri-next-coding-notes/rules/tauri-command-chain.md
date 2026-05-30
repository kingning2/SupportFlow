# Tauri Command Chain

## Rule

需要返回值或强类型错误的 IPC 用 **Command**；跨窗广播、单向通知、前端写 Rust 日志等用 **Event**（见 [tauri-events.md](./tauri-events.md)）。

新增或修改 command 时，必须同步修改以下位置：

1. Rust command 文件（`src-tauri/src/cmd/*.rs`）
2. Rust 模块导出（`src-tauri/src/cmd/mod.rs`）
3. Rust handler 注册（`src-tauri/src/lib.rs`）
4. 前端枚举（`src/enums/tauri-cmd.ts` 增加 `TauriCmd` 成员，字符串与 Rust 命令名一致）
5. 前端封装函数（`src/cmd/*.ts` 使用 `invokeWrapper(TauriCmd.Xxx, …)`）

前端**不要**使用 `src/cmd/types.ts` 或裸字符串命令名（已废弃，统一 `TauriCmd`）。

## Incorrect

只写了 Rust command，前端直接 `invoke('xxx')` 或 `invokeWrapper('xxx')` 硬编码。

## Correct

```ts
import { TauriCmd } from '@/enums'
import { invokeWrapper } from '@/cmd'

export const getAppSession = () => invokeWrapper<AppSession>(TauriCmd.GetAppSession)
```
