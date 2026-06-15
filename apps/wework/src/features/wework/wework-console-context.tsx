"use client";

import { createContext, useContext } from "react";

import type { ChannelCatalogEntry } from "@supportflow/shared";

import type { PageActions } from "./accounts/page-types";
import type { WeworkConnectionStatus } from "./types/wework-conversation";

export interface WeworkConsoleContextValue {
  lang: string;
  actions: PageActions;
  channel: ChannelCatalogEntry | null;
  channelLoading: boolean;
  channelError: string | null;
  connectionStatus: WeworkConnectionStatus;
  onChannelUpdated: () => void;
}

export const WeworkConsoleContext = createContext<WeworkConsoleContextValue | null>(null);

export function useWeworkConsoleContext(): WeworkConsoleContextValue {
  const ctx = useContext(WeworkConsoleContext);
  if (!ctx) {
    throw new Error("useWeworkConsoleContext must be used within <WeworkConsoleLayout />");
  }
  return ctx;
}
