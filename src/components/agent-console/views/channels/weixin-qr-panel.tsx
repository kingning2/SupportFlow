"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Check, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  cowChannelAction,
  fetchCowChannelConsoleApi,
  fetchCowChannels
} from "@/cmd/cow-python-channels";

interface WeixinQrPanelProps {
  mode: "add" | "active";
  onConnected: () => void;
}

export function WeixinQrPanel({ mode, onConnected }: WeixinQrPanelProps) {
  const { t, i18n } = useTranslation("console");
  const lang = i18n.language.startsWith("zh") ? "zh" : "en";
  const [statusText, setStatusText] = useState(t("weixin_scan_loading"));
  const [statusClass, setStatusClass] = useState("text-slate-500 dark:text-slate-400");
  const [qrImage, setQrImage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const pollRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const statusPollRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const stopPoll = useCallback(() => {
    if (pollRef.current) {
      clearTimeout(pollRef.current);
      pollRef.current = null;
    }
    if (statusPollRef.current) {
      clearTimeout(statusPollRef.current);
      statusPollRef.current = null;
    }
  }, []);

  const pollQr = useCallback(() => {
    pollRef.current = setTimeout(async () => {
      try {
        const data = await fetchCowChannelConsoleApi("weixin/qrlogin", "POST", { action: "poll" });
        if (data.status !== "success") {
          pollQr();
          return;
        }
        const qrStatus = data.qr_status as string | undefined;
        if (qrStatus === "confirmed") {
          stopPoll();
          setStatusText(t("weixin_scan_success"));
          setStatusClass("text-[#35A85B]");
          await cowChannelAction({ action: "connect", channel: "weixin", config: {} });
          setTimeout(() => onConnected(), 1500);
          return;
        }
        if (qrStatus === "expired" && (data.qr_image || data.qrcode_url)) {
          setQrImage(String(data.qr_image || data.qrcode_url));
          setStatusText(t("weixin_scan_waiting"));
        } else if (qrStatus === "scaned" || qrStatus === "scanned") {
          setStatusText(t("weixin_scan_scanned"));
          setStatusClass("text-[#35A85B]");
        }
        pollQr();
      } catch {
        pollQr();
      }
    }, 2000);
  }, [onConnected, stopPoll, t]);

  const loadQr = useCallback(async () => {
    setError(null);
    setStatusText(t("weixin_scan_loading"));
    try {
      const data = await fetchCowChannelConsoleApi("weixin/qrlogin", "GET");
      if (data.status !== "success") {
        setError(`${t("weixin_scan_fail")}: ${data.message ?? ""}`);
        return;
      }
      const img = String(data.qr_image || data.qrcode_url || "");
      if (img) {
        setQrImage(img);
        setStatusText(t("weixin_scan_waiting"));
        if (data.source === "channel") {
          const statusPoll = () => {
            statusPollRef.current = setTimeout(async () => {
              try {
                const list = await fetchCowChannels();
                const wx = list.find((c) => c.name === "weixin");
                const st = wx?.login_status ?? wx?.loginStatus;
                if (st === "logged_in") {
                  stopPoll();
                  onConnected();
                  return;
                }
                statusPoll();
              } catch {
                statusPoll();
              }
            }, 3000);
          };
          statusPoll();
        } else {
          pollQr();
        }
      }
    } catch {
      setError(t("weixin_scan_fail"));
    }
  }, [onConnected, pollQr, stopPoll, t]);

  useEffect(() => {
    let cancelled = false;
    queueMicrotask(() => {
      if (!cancelled) void loadQr();
    });
    return () => {
      cancelled = true;
      stopPoll();
    };
  }, [loadQr, stopPoll]);

  if (error) {
    return <p className="text-sm text-red-500">{error}</p>;
  }

  if (!qrImage && mode === "add") {
    return (
      <div className="flex flex-col items-center py-4">
        <Loader2 className="mb-4 size-6 animate-spin text-slate-400" />
        <p className="text-sm text-slate-500 dark:text-slate-400">{statusText}</p>
      </div>
    );
  }

  if (mode === "active" && !qrImage) {
    return (
      <div className="flex flex-col items-center py-2">
        <button
          type="button"
          className="cursor-pointer rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#228547]"
          onClick={() => void loadQr()}
        >
          {t("weixin_scan_title")}
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center py-2">
      <p className="mb-1 text-sm font-medium text-slate-700 dark:text-slate-200">
        {t("weixin_scan_title")}
      </p>
      <p className="mb-4 text-xs text-slate-400 dark:text-slate-500">{t("weixin_scan_desc")}</p>
      {qrImage ? (
        <div className="mb-3 rounded-xl border border-slate-100 bg-white p-3 shadow-sm dark:border-slate-700">
          <img src={qrImage} alt="QR" className="size-52" style={{ imageRendering: "pixelated" }} />
        </div>
      ) : (
        <Check className="mb-3 size-12 text-[#35A85B]" />
      )}
      <p className={`mb-1 text-xs ${statusClass}`}>{statusText}</p>
      <p className="text-xs text-slate-400 dark:text-slate-500">
        {lang === "zh" ? t("weixin_qr_tip") : t("weixin_qr_tip")}
      </p>
    </div>
  );
}
