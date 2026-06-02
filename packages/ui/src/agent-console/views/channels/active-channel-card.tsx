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
import { FeishuPanel } from "./feishu-panel";
import { WecomPanel } from "./wecom-panel";
import { WeixinQrPanel } from "./weixin-qr-panel";
import { WxQrPanel } from "./wx-qr-panel";

function wecomHasCreds(ch: ChannelCatalogEntry) {
  const id = ch.fields.find((f) => f.key === "wecom_bot_id");
  const secret = ch.fields.find((f) => f.key === "wecom_bot_secret");
  return !!(id?.value && secret?.value);
}

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

  const weixinWaiting = channel.name === "weixin" && loginStatus && loginStatus !== "logged_in";
  const wxWaiting = channel.name === "wx" && loginStatus && loginStatus !== "logged_in";
  const wecomNeedsCreds = channel.name === "wecom_bot" && !wecomHasCreds(channel);
  const isFeishu = channel.name === "feishu";
  const hasFields = channel.fields.length > 0;

  let statusDot = "bg-[#4ABE6E]";
  let statusLabel = <span className="text-xs text-[#35A85B]">{t("channels_connected")}</span>;
  if (weixinWaiting || wxWaiting) {
    statusDot = "animate-pulse bg-amber-400";
    statusLabel =
      loginStatus === "scanned" ? (
        <span className="text-xs text-[#35A85B]">{t("weixin_scan_scanned")}</span>
      ) : (
        <span className="text-xs text-amber-500">{t("weixin_scan_waiting")}</span>
      );
  } else if (wecomNeedsCreds) {
    statusDot = "animate-pulse bg-amber-400";
    statusLabel = <span className="text-xs text-amber-500">{t("channels_connecting")}</span>;
  }

  const showSaveBlock = hasFields && !weixinWaiting && !wxWaiting && !wecomNeedsCreds && !isFeishu;

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

  const headerMb =
    hasFields || weixinWaiting || wxWaiting || wecomNeedsCreds || isFeishu ? "mb-5" : "";

  return (
    <div
      id={`channel-card-${channel.name}`}
      className="rounded-xl border border-slate-200 bg-white p-6 dark:border-white/10 dark:bg-[#1A1A1A]"
    >
      <div className={`flex items-center gap-4${headerMb ? ` ${headerMb}` : ""}`}>
        <div
          className={`flex size-10 shrink-0 items-center justify-center rounded-xl ${colors.iconBox}`}
        >
          <Icon className={`size-4 ${colors.icon}`} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-slate-800 dark:text-slate-100">{label}</span>
            <span className={`size-2 rounded-full ${statusDot}`} />
            {statusLabel}
          </div>
          <p className="mt-0.5 font-mono text-xs text-slate-500 dark:text-slate-400">
            {channel.name}
          </p>
        </div>
        <button
          type="button"
          className="shrink-0 cursor-pointer rounded-lg bg-red-50 px-3 py-1.5 text-xs font-medium text-red-500 transition-colors hover:bg-red-100 dark:bg-red-900/20 dark:text-red-400 dark:hover:bg-red-900/40"
          onClick={() => onDisconnect(channel.name)}
        >
          {t("channels_disconnect")}
        </button>
      </div>

      {weixinWaiting ? <WeixinQrPanel mode="active" onConnected={onRefresh} /> : null}
      {wxWaiting ? <WxQrPanel onLoggedIn={onRefresh} /> : null}
      {wecomNeedsCreds ? (
        <WecomPanel channel={channel} lang={lang} variant="active" onConnected={onRefresh} />
      ) : null}
      {isFeishu ? (
        <FeishuPanel
          channel={channel}
          lang={lang}
          isActive
          onConnected={onRefresh}
          onSaveStatus={(key, err) => {
            setStatusMsg(t(key));
            setStatusError(err);
            setTimeout(() => setStatusMsg(null), 2500);
          }}
        />
      ) : null}
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
              } ${statusError ? "text-red-500" : "text-[#35A85B]"}`}
            >
              {statusMsg}
            </span>
            <button
              type="button"
              disabled={saving}
              className="cursor-pointer rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#228547] disabled:cursor-not-allowed disabled:opacity-50"
              onClick={() => void handleSave()}
            >
              {saving ? t("channels_saving") : t("channels_save")}
            </button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
