import { emit, emitTo, listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

import { isTauriRuntime } from "@/cmd";

export type TauriEventHandler<T> = (event: Event<T>) => void;

export type TauriEventScope = "global" | "webview";

/** 前端 → Rust / 其他 Webview（全局广播） */
export async function tauriEmit(event: string, payload?: unknown): Promise<void> {
  if (!isTauriRuntime()) return;
  await emit(event, payload);
}

/** 前端 → 指定 Webview（对应 Rust 的 emit_to） */
export async function tauriEmitTo(target: string, event: string, payload?: unknown): Promise<void> {
  if (!isTauriRuntime()) return;
  await emitTo(target, event, payload);
}

/** 订阅 Rust / 其他 Webview 发来的事件（全局，跨窗 IPC） */
export function tauriOn<T>(event: string, handler: TauriEventHandler<T>): () => void {
  if (!isTauriRuntime()) return () => {};

  let unlisten: UnlistenFn | undefined;

  void listen<T>(event, handler).then((fn) => {
    unlisten = fn;
  });

  return () => {
    unlisten?.();
  };
}

/** 仅订阅发往当前 Webview 的事件（对应 Rust 的 emit_to 目标窗） */
export function tauriOnWebview<T>(event: string, handler: TauriEventHandler<T>): () => void {
  if (!isTauriRuntime()) return () => {};

  let unlisten: UnlistenFn | undefined;

  void getCurrentWebviewWindow()
    .listen<T>(event, handler)
    .then((fn) => {
      unlisten = fn;
    });

  return () => {
    unlisten?.();
  };
}

export function tauriOnScoped<T>(
  event: string,
  handler: TauriEventHandler<T>,
  scope: TauriEventScope = "global"
): () => void {
  return scope === "webview" ? tauriOnWebview(event, handler) : tauriOn(event, handler);
}
