"use client";

import { useTranslation } from "react-i18next";

import { ViewShell } from "@/components/agent-console/shared/console-brand";
import type { AgentConsoleState } from "@/generated/contracts";

export function ConfigView({ state }: { state: AgentConsoleState | null }) {
  const { t } = useTranslation("console");

  return (
    <ViewShell title={t("config_title")} description={t("config_desc_invoke")}>
      <div className="grid gap-6 lg:grid-cols-2">
        <section className="rounded-xl border border-slate-200 p-4 dark:border-white/10">
          <h3 className="mb-3 text-sm font-semibold">{t("config_paths")}</h3>
          <dl className="space-y-2 text-sm">
            <div>
              <dt className="text-muted-foreground">{t("workspace_label")}</dt>
              <dd className="font-mono text-xs break-all">{state?.workspaceDir ?? "—"}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">{t("config_bundled_label")}</dt>
              <dd className="font-mono text-xs break-all">{state?.configPath ?? "—"}</dd>
            </div>
          </dl>
        </section>

        <section className="rounded-xl border border-slate-200 p-4 dark:border-white/10">
          <h3 className="mb-3 text-sm font-semibold">{t("config_sampling")}</h3>
          <dl className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <dt className="text-muted-foreground">{t("config_temperature")}</dt>
              <dd>{state?.temperature ?? t("config_default")}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">top_p</dt>
              <dd>{state?.topP ?? t("config_default")}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">{t("config_timeout")}</dt>
              <dd>{state?.requestTimeout ?? t("config_default")}</dd>
            </div>
          </dl>
        </section>
      </div>

      {state?.mcpStatus && Object.keys(state.mcpStatus).length > 0 ? (
        <section className="mt-6 rounded-xl border border-slate-200 p-4 dark:border-white/10">
          <h3 className="mb-3 text-sm font-semibold">MCP</h3>
          <ul className="space-y-1 text-sm">
            {Object.entries(state.mcpStatus).map(([name, status]) => (
              <li key={name} className="flex justify-between gap-4">
                <span className="font-mono text-xs">{name}</span>
                <span className="text-muted-foreground">{status}</span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <p className="text-muted-foreground mt-6 text-xs">{t("config_edit_hint")}</p>
    </ViewShell>
  );
}
