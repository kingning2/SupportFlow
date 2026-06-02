"use client";

import { MessageCircle } from "lucide-react";
import { useTranslation } from "react-i18next";

/** 微信个人号（wx）独立入口页 — 后续可接入扫码面板 */
export function WechatPage() {
  const { t } = useTranslation("console");

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-y-auto p-6">
        <div className="mx-auto flex max-w-lg flex-col items-center py-20 text-center">
          <div className="mb-4 flex size-16 items-center justify-center rounded-2xl bg-[#07C160]/10">
            <MessageCircle className="size-8 text-[#07C160]" />
          </div>
          <h1 className="text-xl font-bold text-slate-800 dark:text-slate-100">
            {t("channel_label_wx")}
          </h1>
          <p className="mt-2 text-sm text-slate-500 dark:text-slate-400">
            {t("channels_dev_desc")}
          </p>
          <p className="mt-6 rounded-lg border border-dashed border-slate-200 px-4 py-3 text-xs text-slate-400 dark:border-white/10">
            扫码接入 UI 将在本应用内实现，与 wework 应用独立打包。
          </p>
        </div>
      </div>
    </div>
  );
}
