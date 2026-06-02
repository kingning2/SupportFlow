"use client";

import { useEffect, useState } from "react";
import { Brain, FileText, Moon } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  listAgentMemory,
  readAgentMemory,
  type AgentMemoryItem
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { ViewShell } from "../shared/console-brand";
import { Button } from "@supportflow/ui/button";

export function MemoryView() {
  const { t } = useTranslation("console");
  const [loading, setLoading] = useState(true);
  const [items, setItems] = useState<AgentMemoryItem[]>([]);
  const [activeTab, setActiveTab] = useState<"files" | "dreams">("files");
  const [activeFilename, setActiveFilename] = useState<string | null>(null);
  const [activeContent, setActiveContent] = useState("");

  useEffect(() => {
    let mounted = true;
    const load = async () => {
      try {
        const data = await listAgentMemory();
        if (mounted) {
          setItems(data);
        }
      } catch {
        if (mounted) {
          setItems([]);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };
    void load();
    return () => {
      mounted = false;
    };
  }, []);

  const filtered = items.filter((item) =>
    activeTab === "dreams" ? item.type === "dream" : item.type !== "dream"
  );

  const openFile = async (filename: string) => {
    try {
      const result = await readAgentMemory(filename);
      setActiveFilename(result.filename);
      setActiveContent(result.content);
    } catch {
      setActiveFilename(filename);
      setActiveContent("");
    }
  };

  return (
    <ViewShell title={t("memory_title")} description={t("memory_desc")}>
      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-6 flex items-center justify-between gap-3">
          <div className="flex items-center rounded-lg bg-slate-100 p-0.5 dark:bg-white/10">
            <Button
              type="button"
              size="sm"
              className="h-8 px-3 text-xs"
              variant={activeTab === "files" ? "default" : "ghost"}
              onClick={() => setActiveTab("files")}
            >
              <FileText className="mr-1.5 size-3.5" />
              {t("memory_tab_files")}
            </Button>
            <Button
              type="button"
              size="sm"
              className="h-8 px-3 text-xs"
              variant={activeTab === "dreams" ? "default" : "ghost"}
              onClick={() => setActiveTab("dreams")}
            >
              <Moon className="mr-1.5 size-3.5" />
              {t("memory_tab_dreams")}
            </Button>
          </div>
        </div>

        {activeFilename ? (
          <div className="rounded-xl border border-slate-200 dark:border-white/10">
            <div className="border-b border-slate-200 px-4 py-2 font-mono text-sm dark:border-white/10">
              {activeFilename}
            </div>
            <div className="max-h-[60vh] overflow-y-auto p-4">
              <pre className="text-sm whitespace-pre-wrap text-slate-700 dark:text-slate-200">
                {activeContent || t("memory_empty")}
              </pre>
            </div>
            <div className="border-t border-slate-200 p-3 dark:border-white/10">
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => setActiveFilename(null)}
              >
                {t("memory_back")}
              </Button>
            </div>
          </div>
        ) : loading || filtered.length === 0 ? (
          <div className="flex flex-col items-center justify-center rounded-xl border border-slate-200 py-20 dark:border-white/10">
            <div className="mb-4 flex size-16 items-center justify-center rounded-2xl bg-purple-50 dark:bg-purple-900/20">
              <Brain className="size-7 text-purple-400" />
            </div>
            <p className="font-medium text-slate-500 dark:text-slate-400">{t("memory_title")}</p>
            <p className="mt-1 text-sm text-slate-400 dark:text-slate-500">
              {loading ? t("memory_loading_desc") : t("memory_empty")}
            </p>
          </div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-slate-200 dark:border-white/10">
            <table className="w-full">
              <thead>
                <tr className="border-b border-slate-200 dark:border-white/10">
                  <th className="px-4 py-3 text-left text-xs text-slate-500">
                    {t("memory_col_name")}
                  </th>
                  <th className="px-4 py-3 text-left text-xs text-slate-500">
                    {t("memory_col_type")}
                  </th>
                  <th className="px-4 py-3 text-left text-xs text-slate-500">
                    {t("memory_col_size")}
                  </th>
                  <th className="px-4 py-3 text-left text-xs text-slate-500">
                    {t("memory_col_updated")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((item) => (
                  <tr
                    key={item.filename}
                    className="cursor-pointer border-b border-slate-100 hover:bg-slate-50 dark:border-white/5 dark:hover:bg-white/5"
                    onClick={() => void openFile(item.filename)}
                  >
                    <td className="px-4 py-3 font-mono text-sm">{item.filename}</td>
                    <td className="px-4 py-3 text-sm">{item.type}</td>
                    <td className="px-4 py-3 text-sm">{item.size}</td>
                    <td className="px-4 py-3 text-sm">{item.updatedAt}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </ViewShell>
  );
}
