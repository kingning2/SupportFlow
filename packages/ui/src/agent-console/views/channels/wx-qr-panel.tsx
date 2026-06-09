"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Loader2 } from "lucide-react";

import {
  fetchChannelConsoleApi,
  fetchChannels
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";

interface WxQrPanelProps {
  onLoggedIn: () => void;
}

export function WxQrPanel({ onLoggedIn }: WxQrPanelProps) {
  const lang = "zh";
  const [qrImage, setQrImage] = useState<string | null>(null);
  const [statusText, setStatusText] = useState("正在获取二维码…");
  const [statusClass, setStatusClass] = useState("text-slate-500 dark:text-slate-400");
  const pollRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const statusPollRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const stopAll = useCallback(() => {
    if (pollRef.current) {
      clearTimeout(pollRef.current);
      pollRef.current = null;
    }
    if (statusPollRef.current) {
      clearTimeout(statusPollRef.current);
      statusPollRef.current = null;
    }
  }, []);

  const pollWx = useCallback(() => {
    pollRef.current = setTimeout(async () => {
      try {
        const data = await fetchChannelConsoleApi("wx/qrlogin", "POST", { action: "poll" });
        if (data.status !== "success") {
          pollWx();
          return;
        }
        if (data.qr_status === "confirmed" || data.login_status === "logged_in") {
          stopAll();
          setStatusText("登录成功，正在启动通道…");
          setStatusClass("text-[#35A85B]");
          onLoggedIn();
          return;
        }
        const img = String(data.qr_image || data.qrcode_url || "");
        if (img) {
          setQrImage(img);
          const scanned = data.qr_status === "scaned" || data.login_status === "scanned";
          setStatusText(scanned ? "已扫码，请在手机上确认" : "等待扫码…");
          setStatusClass(scanned ? "text-[#35A85B]" : "text-slate-500 dark:text-slate-400");
        }
        pollWx();
      } catch {
        pollWx();
      }
    }, 2000);
  }, [onLoggedIn, stopAll]);

  const loadQr = useCallback(async () => {
    setStatusText("正在获取二维码…");
    try {
      const data = await fetchChannelConsoleApi("wx/qrlogin", "GET");
      if (data.status !== "success") {
        setStatusText(`获取二维码失败: ${data.message ?? ""}`);
        setStatusClass("text-red-500");
        return;
      }
      if (data.qr_status === "confirmed" || data.login_status === "logged_in") {
        onLoggedIn();
        return;
      }
      const img = String(data.qr_image || data.qrcode_url || "");
      if (img) {
        setQrImage(img);
        setStatusText("等待扫码…");
        pollWx();
      }
    } catch {
      setStatusText("获取二维码失败");
      setStatusClass("text-red-500");
    }
  }, [onLoggedIn, pollWx]);

  const startStatusPoll = useCallback(() => {
    statusPollRef.current = setTimeout(async () => {
      try {
        const channels = await fetchChannels();
        const row = channels.find((c) => c.name === "wx");
        if (row && (row.login_status === "logged_in" || row.loginStatus === "logged_in")) {
          stopAll();
          onLoggedIn();
          return;
        }
        startStatusPoll();
      } catch {
        startStatusPoll();
      }
    }, 3000);
  }, [onLoggedIn, stopAll]);

  useEffect(() => {
    let cancelled = false;
    queueMicrotask(() => {
      if (!cancelled) {
        void loadQr();
        startStatusPoll();
      }
    });
    return () => {
      cancelled = true;
      stopAll();
    };
  }, [loadQr, startStatusPoll, stopAll]);

  return (
    <div className="flex flex-col items-center py-2">
      {!qrImage ? (
        <Loader2 className="mb-4 size-6 animate-spin text-slate-400" />
      ) : (
        <>
          <p className="mb-1 text-sm font-medium text-slate-700 dark:text-slate-200">
            {"微信扫码登录"}
          </p>
          <p className="mb-4 text-xs text-slate-400 dark:text-slate-500">
            {lang === "zh" ? "个人微信 itchat 扫码" : "Personal WeChat (itchat) QR"}
          </p>
          <div className="mb-3 rounded-xl border border-slate-100 bg-white p-3 shadow-sm dark:border-slate-700">
            <img
              src={qrImage}
              alt="QR"
              className="size-52"
              style={{ imageRendering: "pixelated" }}
            />
          </div>
        </>
      )}
      <p className={`mb-1 text-xs ${statusClass}`}>{statusText}</p>
      <p className="text-xs text-slate-400 dark:text-slate-500">{"二维码约 2 分钟后过期"}</p>
    </div>
  );
}
