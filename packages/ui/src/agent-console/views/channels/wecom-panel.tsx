"use client";

import { useCallback, useState } from "react";
import { Check, QrCode } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  channelAction,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import {
  buildConfigFromDrafts,
  ChannelFields,
  draftsFromChannel,
  type ChannelFieldDrafts
} from "./channel-fields";
import { channelModeTabClass, wecomHasCreds } from "./channel-utils";

const WECOM_BOT_SDK_URL = "https://wwcdn.weixin.qq.com/node/wework/js/wecom-aibot-sdk@0.1.0.min.js";
const WECOM_BOT_SOURCE = "SupportFlow";

declare global {
  interface Window {
    WecomAIBotSDK?: {
      openBotInfoAuthWindow: (opts: {
        source: string;
        onCreated: (bot: { botid: string; secret: string }) => void;
        onError: (err: { message?: string; code?: string }) => void;
      }) => void;
    };
  }
}

function ensureWecomSdk(): Promise<void> {
  return new Promise((resolve, reject) => {
    if (window.WecomAIBotSDK) {
      resolve();
      return;
    }
    const existing = document.querySelector(`script[src="${WECOM_BOT_SDK_URL}"]`);
    if (existing) {
      resolve();
      return;
    }
    const s = document.createElement("script");
    s.src = WECOM_BOT_SDK_URL;
    s.onload = () => resolve();
    s.onerror = () => reject(new Error("Failed to load WecomAIBotSDK"));
    document.head.appendChild(s);
  });
}

interface WecomPanelProps {
  channel: ChannelCatalogEntry;
  lang: string;
  variant: "add" | "active";
  onConnected: () => void;
  showManualActions?: boolean;
  onManualConnect?: () => void;
}

export function WecomPanel({
  channel,
  lang,
  variant,
  onConnected,
  showManualActions = false,
  onManualConnect
}: WecomPanelProps) {
  const { t } = useTranslation("console");
  const [modeOverride, setModeOverride] = useState<"scan" | "manual" | null>(null);
  const mode = modeOverride ?? (wecomHasCreds(channel) ? "manual" : "scan");
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() => draftsFromChannel(channel));
  const [scanStatus, setScanStatus] = useState<"idle" | "ok" | "error">("idle");
  const [scanMessage, setScanMessage] = useState("");

  const connectAfterAuth = useCallback(
    async (botId: string, secret: string) => {
      await channelAction({
        action: "connect",
        channel: "wecom_bot",
        config: { wecom_bot_id: botId, wecom_bot_secret: secret }
      });
      setTimeout(() => onConnected(), 1500);
    },
    [onConnected]
  );

  const startAuth = useCallback(async () => {
    setScanStatus("idle");
    setScanMessage("");
    try {
      await ensureWecomSdk();
      window.WecomAIBotSDK?.openBotInfoAuthWindow({
        source: WECOM_BOT_SOURCE,
        onCreated: (bot) => {
          setScanStatus("ok");
          void connectAfterAuth(bot.botid, bot.secret);
        },
        onError: (err) => {
          setScanStatus("error");
          setScanMessage(`${t("wecom_scan_fail")}: ${err.message ?? err.code ?? ""}`);
        }
      });
    } catch (e) {
      setScanStatus("error");
      setScanMessage(e instanceof Error ? e.message : "SDK load failed");
    }
  }, [connectAfterAuth, t]);

  if (variant === "active" && !wecomHasCreds(channel)) {
    return (
      <div className="flex flex-col items-center py-2">
        <p className="mb-3 text-sm text-slate-500 dark:text-slate-400">{t("wecom_scan_desc")}</p>
        <button
          type="button"
          className="flex cursor-pointer items-center rounded-lg bg-emerald-500 px-5 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-600"
          onClick={() => void startAuth()}
        >
          <QrCode className="mr-2 size-4" />
          {t("wecom_scan_btn")}
        </button>
        {scanStatus === "ok" ? (
          <p className="mt-3 text-sm font-medium text-emerald-600 dark:text-emerald-400">
            {t("wecom_scan_success")}
          </p>
        ) : null}
        {scanStatus === "error" ? <p className="mt-3 text-sm text-red-500">{scanMessage}</p> : null}
      </div>
    );
  }

  return (
    <div>
      <div className="mb-5 flex items-center justify-center gap-1 rounded-lg bg-slate-100 p-1 dark:bg-white/5">
        <button
          type="button"
          className={channelModeTabClass(mode === "scan")}
          onClick={() => setModeOverride("scan")}
        >
          {t("wecom_mode_scan")}
        </button>
        <button
          type="button"
          className={channelModeTabClass(mode === "manual")}
          onClick={() => setModeOverride("manual")}
        >
          {t("wecom_mode_manual")}
        </button>
      </div>

      {mode === "scan" ? (
        <div className="flex flex-col items-center py-4">
          <p className="mb-2 text-sm text-slate-600 dark:text-slate-300">{t("wecom_scan_desc")}</p>
          <button
            type="button"
            className="mt-3 flex cursor-pointer items-center rounded-lg bg-emerald-500 px-6 py-2.5 text-sm font-medium text-white transition-colors hover:bg-emerald-600"
            onClick={() => void startAuth()}
          >
            <QrCode className="mr-2 size-4" />
            {t("wecom_scan_btn")}
          </button>
          {scanStatus === "ok" ? (
            <div className="mt-3 flex flex-col items-center py-2">
              <div className="mb-2 flex size-10 items-center justify-center rounded-full bg-emerald-50 dark:bg-emerald-900/30">
                <Check className="size-5 text-emerald-500" />
              </div>
              <p className="text-sm font-medium text-emerald-600 dark:text-emerald-400">
                {t("wecom_scan_success")}
              </p>
            </div>
          ) : null}
          {scanStatus === "error" ? (
            <p className="mt-3 text-sm text-red-500">{scanMessage}</p>
          ) : null}
        </div>
      ) : (
        <div className="space-y-4">
          <ChannelFields
            channelName="wecom_bot"
            fields={channel.fields}
            lang={lang}
            drafts={drafts}
            onChange={setDrafts}
          />
          {showManualActions && onManualConnect ? (
            <div className="flex justify-end pt-2">
              <button
                type="button"
                className="cursor-pointer rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white hover:bg-[#228547]"
                onClick={() => {
                  void channelAction({
                    action: "connect",
                    channel: "wecom_bot",
                    config: buildConfigFromDrafts(channel, drafts)
                  }).then(() => onManualConnect());
                }}
              >
                {t("channels_connect_btn")}
              </button>
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
}
