"use client";

import { useEffect, useState } from "react";
import { Clock3 } from "lucide-react";
import { useTranslation } from "react-i18next";

import { listAgentTasks, type AgentTaskSummary } from "@supportflow/shared/tauri-bridge/cmd/agent";
import { ViewShell } from "../shared/console-brand";

export function TasksView() {
  const { t } = useTranslation("console");
  const [loading, setLoading] = useState(true);
  const [tasks, setTasks] = useState<AgentTaskSummary[]>([]);

  useEffect(() => {
    let mounted = true;
    const load = async () => {
      try {
        const data = await listAgentTasks();
        if (mounted) {
          setTasks(data.filter((task) => task.enabled !== false));
        }
      } catch {
        if (mounted) {
          setTasks([]);
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

  return (
    <ViewShell title={t("tasks_title")} description={t("tasks_desc")}>
      <div className="mx-auto w-full max-w-4xl">
        {loading || tasks.length === 0 ? (
          <div className="flex flex-col items-center justify-center rounded-xl border border-slate-200 py-20 dark:border-white/10">
            <div className="mb-4 flex size-16 items-center justify-center rounded-2xl bg-rose-50 dark:bg-rose-900/20">
              <Clock3 className="size-7 text-rose-400" />
            </div>
            <p className="font-medium text-slate-500 dark:text-slate-400">{t("tasks_title")}</p>
            <p className="mt-1 text-sm text-slate-400 dark:text-slate-500">
              {loading ? t("tasks_loading") : t("tasks_empty")}
            </p>
          </div>
        ) : (
          <div className="grid gap-4">
            {tasks.map((task) => (
              <div
                key={task.id}
                className="rounded-xl border border-slate-200 p-4 dark:border-white/10 dark:bg-[#1A1A1A]"
              >
                <div className="mb-2 flex items-center gap-2">
                  <span className="size-2 rounded-full bg-[#35A85B]" />
                  <span className="text-sm font-medium text-slate-700 dark:text-slate-200">
                    {task.name || task.id}
                  </span>
                </div>
                <p className="text-xs text-slate-500 dark:text-slate-400">
                  {t("tasks_next_run")}: {task.nextRunAt ?? "—"}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </ViewShell>
  );
}
