"use client";

import { useState } from "react";
import { Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  buildConfigFromDrafts,
  ChannelFields,
  ChannelHint,
  draftsFromChannel,
  type ChannelCatalogEntry,
  type ChannelFieldDrafts
} from "@supportflow/shared";

export interface WeworkConnectPanelProps {
  channel: ChannelCatalogEntry;
  lang: string;
  connecting?: boolean;
  onConnect: (config: Record<string, string | number | boolean>) => void | Promise<void>;
}

/** 企微个人号接入表单（控制台内嵌 + 独立渠道页共用） */
export function WeworkConnectPanel({
  channel,
  lang,
  connecting = false,
  onConnect
}: WeworkConnectPanelProps) {
  const { t } = useTranslation("console");
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() => draftsFromChannel(channel));

  return (
    <div className="space-y-4">
      {channel.hint ? <ChannelHint hint={channel.hint} lang={lang} /> : null}
      <p className="text-xs text-slate-500 dark:text-slate-400">{t("wework_connect_note")}</p>
      <ChannelFields
        channelName="wework"
        fields={channel.fields}
        lang={lang}
        drafts={drafts}
        onChange={setDrafts}
      />
      <div className="flex justify-end pt-2">
        <button
          type="button"
          disabled={connecting}
          className="flex cursor-pointer items-center rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#228547] disabled:cursor-not-allowed disabled:opacity-50"
          onClick={() => void onConnect(buildConfigFromDrafts(channel, drafts))}
        >
          {connecting ? (
            <>
              <Loader2 className="mr-2 size-4 animate-spin" />
              {t("channels_connecting")}
            </>
          ) : (
            t("channels_connect_btn")
          )}
        </button>
      </div>
    </div>
  );
}
