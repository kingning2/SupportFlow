"use client";

import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { channelAction, channelLangFromI18n, fetchChannels } from "@supportflow/ui/app-shell";

import { WeworkConsoleApp } from "@/console/wework-console-app";

export default function WeworkAppPage() {
  const { i18n } = useTranslation("console");
  const lang = channelLangFromI18n(i18n.language);

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
      }
    }),
    []
  );

  return <WeworkConsoleApp lang={lang} actions={actions} />;
}
