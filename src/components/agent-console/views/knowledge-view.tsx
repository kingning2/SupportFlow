"use client";

import { BookOpen, CloudUpload, FolderTree, Network } from "lucide-react";
import { useTranslation } from "react-i18next";

import { ViewShell } from "@/components/agent-console/shared/console-brand";
import { Button } from "@/components/ui/button";

export function KnowledgeView() {
  const { t } = useTranslation("console");

  return (
    <ViewShell title={t("knowledge_title")} description={t("knowledge_desc")}>
      <div className="mx-auto flex w-full max-w-[1600px] flex-col gap-4">
        <div className="flex flex-col justify-between gap-3 sm:flex-row sm:items-center">
          <div />
          <div className="flex items-center gap-2">
            <Button
              type="button"
              size="sm"
              className="h-8 bg-emerald-500 px-3 text-xs hover:bg-emerald-600"
            >
              <CloudUpload className="mr-1.5 size-3.5" />
              {t("knowledge_upload_btn")}
            </Button>
            <div className="flex items-center rounded-lg bg-slate-100 p-0.5 dark:bg-white/10">
              <Button
                type="button"
                size="sm"
                className="h-8 bg-white px-3 text-xs dark:bg-[#1A1A1A]"
              >
                <FolderTree className="mr-1.5 size-3.5" />
                {t("knowledge_tab_docs")}
              </Button>
              <Button type="button" size="sm" variant="ghost" className="h-8 px-3 text-xs">
                <Network className="mr-1.5 size-3.5" />
                {t("knowledge_tab_graph")}
              </Button>
            </div>
          </div>
        </div>

        <div className="flex flex-col items-center justify-center rounded-xl border border-slate-200 py-20 dark:border-white/10">
          <div className="mb-4 flex size-16 items-center justify-center rounded-2xl bg-emerald-50 dark:bg-emerald-900/20">
            <BookOpen className="size-7 text-emerald-400" />
          </div>
          <p className="font-medium text-slate-500 dark:text-slate-400">{t("knowledge_title")}</p>
          <p className="mt-1 text-sm text-slate-400 dark:text-slate-500">
            {t("knowledge_loading_desc")}
          </p>
        </div>
      </div>
    </ViewShell>
  );
}
