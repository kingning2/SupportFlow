"use client";

import { useEffect } from "react";
import { useStore } from "react-redux";

import type { AppSession } from "@/cmd/session";
import { isLanguage, TauriEvent } from "@/enums";
import type { AppStore } from "@/store";
import { changeCurrentLanguageAction } from "@/store/modules/app";
import { tauriOn } from "@/utils/tauri-event";

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
