"use client";

import { useMemo } from "react";

import { channelAction, fetchChannelConsoleApi, fetchChannels } from "@supportflow/ui/app-shell";

import { App } from "@/features/wework/app";

export function WeworkAppPage() {
  const lang = "zh";

  const actions = useMemo(
    () => ({
      fetchChannels,
      connect: async (config: Record<string, string | number | boolean>) => {
        await channelAction({ action: "connect", channel: "wework", config });
      },
      disconnect: async () => {
        await channelAction({ action: "disconnect", channel: "wework" });
      },
      save: async (config: Record<string, string | number | boolean>) => {
        await channelAction({ action: "save", channel: "wework", config });
      },
      syncContacts: async () => {
        await fetchChannelConsoleApi("wework/contacts_sync", "POST", { action: "start" });
      }
    }),
    []
  );

  return <App lang={lang} actions={actions} />;
}
