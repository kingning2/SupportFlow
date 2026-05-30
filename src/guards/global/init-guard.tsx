"use client";

import { useEffect } from "react";
import { useStore } from "react-redux";

import { getAppSession } from "@/cmd/session";
import type { AppStore } from "@/store";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { changeInitializedAction } from "@/store/modules/app";
import { applySessionToStore } from "@/events/cross-webview-sync";

export default function InitGuard({ children }: { children: React.ReactNode }) {
  const dispatch = useAppDispatch();
  const store = useStore() as AppStore;
  const initialized = useAppSelector((state) => state.app.initialized);

  useEffect(() => {
    let cancelled = false;

    getAppSession()
      .then((session) => {
        if (!cancelled) {
          applySessionToStore(store, session);
        }
      })
      .catch(() => {
        /* 非 Tauri：沿用 Redux 默认语言 */
      })
      .finally(() => {
        if (!cancelled) {
          dispatch(changeInitializedAction(true));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [dispatch, store]);

  if (!initialized) return null;

  return <>{children}</>;
}
