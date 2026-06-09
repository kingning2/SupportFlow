"use client";

import { useTranslation } from "react-i18next";

export function ConfigPlaceholderView({ labelKey }: { labelKey: string }) {
  const { t } = useTranslation("console");

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-auto p-6">
        <h2 className="text-lg font-semibold text-[#1A2B4A]">{t(labelKey)}</h2>
        <p className="mt-2 text-sm text-slate-500">{"该页面将接入 CowAgent 配置；当前为占位。"}</p>
      </div>
    </div>
  );
}
