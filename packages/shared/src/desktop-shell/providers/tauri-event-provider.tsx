"use client";

import { createContext, useContext, useMemo } from "react";

import type {
  TauriEventHandler,
  TauriEventScope
} from "@supportflow/shared/tauri-bridge/tauri-event";
import {
  tauriEmit,
  tauriEmitTo,
  tauriOn,
  tauriOnScoped,
  tauriOnWebview
} from "@supportflow/shared/tauri-bridge/tauri-event";

export type TauriEventApi = {
  /** 前端 -> Rust / 全局广播 */
  emit: typeof tauriEmit;
  /** 前端 -> 指定 Webview */
  emitTo: typeof tauriEmitTo;
  /** 监听全局事件（跨 Webview） */
  on: <T>(event: string, handler: TauriEventHandler<T>) => () => void;
  /** 仅监听发往当前 Webview 的事件 */
  onWebview: <T>(event: string, handler: TauriEventHandler<T>) => () => void;
  onScoped: <T>(
    event: string,
    handler: TauriEventHandler<T>,
    scope?: TauriEventScope
  ) => () => void;
};

const TauriEventContext = createContext<TauriEventApi | null>(null);

export default function TauriEventProvider({ children }: { children: React.ReactNode }) {
  const api = useMemo<TauriEventApi>(
    () => ({
      emit: tauriEmit,
      emitTo: tauriEmitTo,
      on: tauriOn,
      onWebview: tauriOnWebview,
      onScoped: tauriOnScoped
    }),
    []
  );

  return <TauriEventContext.Provider value={api}>{children}</TauriEventContext.Provider>;
}

export function useTauriEventApi(): TauriEventApi {
  const ctx = useContext(TauriEventContext);
  if (!ctx) {
    throw new Error("useTauriEventApi must be used within TauriEventProvider");
  }
  return ctx;
}
