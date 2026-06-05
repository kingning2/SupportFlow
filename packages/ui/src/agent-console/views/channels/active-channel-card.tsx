"use client";

import { useState } from "react";
import { MessageCircle } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  channelAction,
  channelLoginStatus,
  localizeChannelText,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import { ChannelHint } from "./channel-hint";
import {
  buildConfigFromDrafts,
  ChannelFields,
  draftsFromChannel,
  type ChannelFieldDrafts
} from "./channel-fields";
import { CHANNEL_ICON_MAP, channelColorClasses } from "./channel-theme";
import { WxQrPanel } from "./wx-qr-panel";
import { Button } from "@supportflow/ui/button";

interface ActiveChannelCardProps {
  channel: ChannelCatalogEntry;
  lang: string;
  onRefresh: () => void;
  onDisconnect: (name: string) => void;
}

export function ActiveChannelCard({
  channel,
  lang,
  onRefresh,
  onDisconnect
}: ActiveChannelCardProps) {
  const { t } = useTranslation("console");
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() => draftsFromChannel(channel));
  const [saving, setSaving] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [statusError, setStatusError] = useState(false);

  const loginStatus = channelLoginStatus(channel);
  const colors = channelColorClasses(channel.color);
  const Icon = CHANNEL_ICON_MAP[channel.icon ?? ""] ?? MessageCircle;
  const label = localizeChannelText(channel.label, lang);

  const wxWaiting = channel.name === "wx" && loginStatus && loginStatus !== "logged_in";
  const hasFields = channel.fields.length > 0;

  let statusDot = "bg-success";
  let statusLabel = <span className="text-success text-xs">{t("channels_connected")}</span>;
  if (wxWaiting) {
    statusDot = "animate-pulse bg-amber-400";
    statusLabel =
      loginStatus === "scanned" ? (
        <span className="text-success text-xs">{t("weixin_scan_scanned")}</span>
      ) : (
        <span className="text-xs text-amber-500">{t("weixin_scan_waiting")}</span>
      );
  }

  const showSaveBlock = hasFields && !wxWaiting;

  const handleSave = async () => {
    setSaving(true);
    try {
      const data = await channelAction({
        action: "save",
        channel: channel.name,
        config: buildConfigFromDrafts(channel, drafts)
      });
      const key = data.restarted ? "channels_restarted" : "channels_saved";
      setStatusMsg(t(key));
      setStatusError(false);
      setTimeout(() => setStatusMsg(null), 2500);
      onRefresh();
    } catch {
      setStatusMsg(t("channels_save_error"));
      setStatusError(true);
      setTimeout(() => setStatusMsg(null), 2500);
    } finally {
      setSaving(false);
    }
  };

  const headerMb = hasFields || wxWaiting ? "mb-5" : "";

  return (
    <div
      id={`channel-card-${channel.name}`}
      className="bg-card border-border rounded-xl border p-6"
    >
      <div className={`flex items-center gap-4${headerMb ? ` ${headerMb}` : ""}`}>
        <div
          className={`flex size-10 shrink-0 items-center justify-center rounded-xl ${colors.iconBox}`}
        >
          <Icon className={`size-4 ${colors.icon}`} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-foreground font-semibold">{label}</span>
            <span className={`size-2 rounded-full ${statusDot}`} />
            {statusLabel}
          </div>
          <p className="text-muted-foreground mt-0.5 font-mono text-xs">{channel.name}</p>
        </div>
        <Button
          type="button"
          variant="destructive"
          className="h-auto px-3 py-1.5 text-xs"
          onClick={() => onDisconnect(channel.name)}
        >
          {t("channels_disconnect")}
        </Button>
      </div>

      {wxWaiting ? <WxQrPanel onLoggedIn={onRefresh} /> : null}
      {showSaveBlock ? (
        <div className="space-y-4">
          {channel.hint ? <ChannelHint hint={channel.hint} lang={lang} /> : null}
          <ChannelFields
            channelName={channel.name}
            fields={channel.fields}
            lang={lang}
            drafts={drafts}
            onChange={setDrafts}
          />
          <div className="flex items-center justify-end gap-3 pt-1">
            <span
              className={`text-xs transition-opacity duration-300 ${
                statusMsg ? "opacity-100" : "opacity-0"
              } ${statusError ? "text-destructive" : "text-success"}`}
            >
              {statusMsg}
            </span>
            <Button type="button" disabled={saving} onClick={() => void handleSave()}>
              {saving ? t("channels_saving") : t("channels_save")}
            </Button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
