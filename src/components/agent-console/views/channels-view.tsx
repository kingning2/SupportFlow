"use client";

import { Plus, RadioTower } from "lucide-react";
import { useTranslation } from "react-i18next";

import { ViewShell } from "@/components/agent-console/shared/console-brand";
import { Button } from "@/components/ui/button";

export function ChannelsView() {
  const { t } = useTranslation("console");

  return (
    <ViewShell title={t("channels_title")} description={t("channels_desc")}>
      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-6 flex items-center justify-end">
          <Button type="button" size="sm" className="bg-[#35A85B] text-white hover:bg-[#228547]">
            <Plus className="mr-1.5 size-4" />
            {t("channels_add")}
          </Button>
        </div>

        <div className="flex flex-col items-center justify-center rounded-xl border border-slate-200 py-20 dark:border-white/10">
          <div className="mb-4 flex size-16 items-center justify-center rounded-2xl bg-blue-50 dark:bg-blue-900/20">
            <RadioTower className="size-7 text-blue-400" />
          </div>
          <p className="font-medium text-slate-500 dark:text-slate-400">{t("channels_empty")}</p>
          <p className="mt-1 text-sm text-slate-400 dark:text-slate-500">
            {t("channels_empty_desc")}
          </p>
        </div>
      </div>
    </ViewShell>
  );
}
