"use client";

import { useEffect } from "react";
import { useStore } from "react-redux";

import type { AppSession } from "@supportflow/shared/tauri-bridge/cmd/session";
import { isLanguage, TauriEvent } from "@supportflow/shared/tauri-bridge/enums";
import type { AppStore } from "../store";
import { changeCurrentLanguageAction } from "../store/modules/app";
import { tauriOn } from "@supportflow/shared/tauri-bridge/tauri-event";

export function applySessionToStore(store: AppStore, session: AppSession) {
  const lang = session.currentLanguage;
  if (isLanguage(lang)) {
    store.dispatch(changeCurrentLanguageAction(lang));
  }
}

function useSessionChangedSubscription() {
  const store = useStore() as AppStore;

  useEffect(() => {
    return tauriOn<AppSession>(TauriEvent.SessionChanged, (event) => {
      applySessionToStore(store, event.payload);
    });
  }, [store]);
}

/** 在 `TauriEventProvider` 内挂载的跨 Webview 订阅（会话同步） */
export function CrossWebviewSyncSubscriptions() {
  useSessionChangedSubscription();
  return null;
}
