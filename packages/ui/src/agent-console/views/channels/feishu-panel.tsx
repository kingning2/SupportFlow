"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Check, QrCode } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  channelAction,
  channelFieldValueString,
  fetchChannelConsoleApi,
  isChannelMaskedSecret,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import {
  buildConfigFromDrafts,
  ChannelFields,
  draftsFromChannel,
  type ChannelFieldDrafts
} from "./channel-fields";

interface FeishuPanelProps {
  channel: ChannelCatalogEntry;
  lang: string;
  isActive?: boolean;
  onSaveStatus?: (msgKey: string, isError: boolean) => void;
  onConnected: () => void;
  showConnectButton?: boolean;
}

export function FeishuPanel({
  channel,
  lang,
  isActive = false,
  onSaveStatus,
  onConnected,
  showConnectButton = false
}: FeishuPanelProps) {
  const { t } = useTranslation("console");
  const [mode, setMode] = useState<"scan" | "manual">(() =>
    channel.fields.some(
      (f) =>
        (f.key === "feishu_app_id" || f.key === "feishu_app_secret") &&
        channelFieldValueString(f.value) &&
        !isChannelMaskedSecret(channelFieldValueString(f.value))
    )
      ? "manual"
      : "scan"
  );
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() => draftsFromChannel(channel));
  const [scanHtml, setScanHtml] = useState<"idle" | "loading" | "qr" | "success" | "error">("idle");
  const [qrImage, setQrImage] = useState("");
  const [qrUrl, setQrUrl] = useState("");
  const [scanError, setScanError] = useState("");
  const [saving, setSaving] = useState(false);
  const pollRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const stopPoll = useCallback(() => {
    if (pollRef.current) {
      clearTimeout(pollRef.current);
      pollRef.current = null;
    }
  }, []);

  const connectAfterRegister = useCallback(
    async (appId: string, appSecret: string) => {
      await channelAction({
        action: "connect",
        channel: "feishu",
        config: { feishu_app_id: appId, feishu_app_secret: appSecret }
      });
      setTimeout(() => onConnected(), 1500);
    },
    [onConnected]
  );

  const pollRegister = useCallback(() => {
    pollRef.current = setTimeout(async () => {
      try {
        const data = await fetchChannelConsoleApi("feishu/register", "POST", { action: "poll" });
        if (data.status !== "success") {
          setScanError((data.message as string) ?? t("feishu_scan_fail"));
          setScanHtml("error");
          return;
        }
        const rs = data.register_status as string;
        if (rs === "done") {
          stopPoll();
          setScanHtml("success");
          await connectAfterRegister(String(data.app_id ?? ""), String(data.app_secret ?? ""));
        } else if (rs === "expired") {
          setScanError(t("feishu_scan_expired"));
          setScanHtml("error");
        } else if (rs === "denied") {
          setScanError(t("feishu_scan_denied"));
          setScanHtml("error");
        } else if (rs === "error") {
          setScanError(String(data.message ?? t("feishu_scan_fail")));
          setScanHtml("error");
        } else {
          pollRegister();
        }
      } catch {
        pollRegister();
      }
    }, 2000);
  }, [connectAfterRegister, stopPoll, t]);

  const startRegister = useCallback(async () => {
    stopPoll();
    setScanHtml("loading");
    setScanError("");
    try {
      const data = await fetchChannelConsoleApi("feishu/register", "GET");
      if (data.status !== "success") {
        setScanError(String(data.message ?? t("feishu_scan_fail")));
        setScanHtml("error");
        return;
      }
      setQrImage(String(data.qr_image ?? ""));
      setQrUrl(String(data.qrcode_url ?? ""));
      setScanHtml("qr");
      pollRegister();
    } catch (e) {
      setScanError(e instanceof Error ? e.message : t("feishu_scan_fail"));
      setScanHtml("error");
    }
  }, [pollRegister, stopPoll, t]);

  useEffect(() => () => stopPoll(), [stopPoll]);

  const tabClass = (active: boolean) =>
    active
      ? "flex-1 rounded-md bg-white px-3 py-1.5 text-xs font-medium text-slate-800 shadow-sm dark:bg-slate-700 dark:text-slate-100"
      : "flex-1 rounded-md px-3 py-1.5 text-xs font-medium text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200";

  const handleSave = async () => {
    setSaving(true);
    try {
      const data = await channelAction({
        action: "save",
        channel: "feishu",
        config: buildConfigFromDrafts(channel, drafts)
      });
      if (data.status === "success") {
        onSaveStatus?.(data.restarted ? "channels_restarted" : "channels_saved", false);
      } else {
        onSaveStatus?.("channels_save_error", true);
      }
    } catch {
      onSaveStatus?.("channels_save_error", true);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div>
      <div className="mb-5 flex items-center justify-center gap-1 rounded-lg bg-slate-100 p-1 dark:bg-white/5">
        <button type="button" className={tabClass(mode === "scan")} onClick={() => setMode("scan")}>
          {t("feishu_mode_scan")}
        </button>
        <button
          type="button"
          className={tabClass(mode === "manual")}
          onClick={() => setMode("manual")}
        >
          {t("feishu_mode_manual")}
        </button>
      </div>

      {mode === "scan" ? (
        <div className="flex flex-col items-center py-4">
          <p className="mb-3 text-center text-sm text-slate-600 dark:text-slate-300">
            {isActive ? t("feishu_scan_replace_desc") : t("feishu_scan_desc")}
          </p>
          {scanHtml === "idle" || scanHtml === "error" ? (
            <>
              {scanError ? (
                <p className="mb-3 text-center text-sm text-red-500">{scanError}</p>
              ) : null}
              <button
                type="button"
                className="mt-2 flex cursor-pointer items-center rounded-lg bg-emerald-500 px-6 py-2.5 text-sm font-medium text-white transition-colors hover:bg-emerald-600"
                onClick={() => void startRegister()}
              >
                <QrCode className="mr-2 size-4" />
                {scanHtml === "error" ? t("feishu_scan_retry") : t("feishu_scan_btn")}
              </button>
            </>
          ) : null}
          {scanHtml === "loading" ? (
            <p className="text-sm text-slate-500 dark:text-slate-400">{t("feishu_scan_loading")}</p>
          ) : null}
          {scanHtml === "qr" ? (
            <div className="flex flex-col items-center gap-3">
              {qrImage ? (
                <img
                  src={qrImage}
                  alt="QR"
                  className="size-44 rounded-lg border border-slate-200 bg-white p-2 dark:border-white/10"
                />
              ) : null}
              <p className="text-xs text-amber-500">{t("feishu_scan_waiting")}</p>
              <p className="text-xs text-slate-400 dark:text-slate-500">{t("feishu_scan_tip")}</p>
              {qrUrl ? (
                <a
                  href={qrUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs text-blue-500 underline hover:text-blue-600"
                >
                  {t("feishu_scan_open_link")}
                </a>
              ) : null}
            </div>
          ) : null}
          {scanHtml === "success" ? (
            <div className="flex flex-col items-center py-2">
              <div className="mb-2 flex size-10 items-center justify-center rounded-full bg-emerald-50 dark:bg-emerald-900/30">
                <Check className="size-5 text-emerald-500" />
              </div>
              <p className="text-sm font-medium text-emerald-600 dark:text-emerald-400">
                {t("feishu_scan_success")}
              </p>
            </div>
          ) : null}
        </div>
      ) : (
        <div className="space-y-4">
          <ChannelFields
            channelName="feishu"
            fields={channel.fields}
            lang={lang}
            drafts={drafts}
            onChange={setDrafts}
          />
          {isActive || showConnectButton ? (
            <div className="flex items-center justify-end gap-3 pt-1">
              <button
                type="button"
                disabled={saving}
                className="cursor-pointer rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#228547] disabled:opacity-50"
                onClick={() =>
                  void (isActive
                    ? handleSave()
                    : channelAction({
                        action: "connect",
                        channel: "feishu",
                        config: buildConfigFromDrafts(channel, drafts)
                      }).then(() => onConnected()))
                }
              >
                {saving
                  ? t("channels_saving")
                  : isActive
                    ? t("channels_save")
                    : t("channels_connect_btn")}
              </button>
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
}
