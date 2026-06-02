"use client";

import { RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";

import { refreshAgentSkills } from "@supportflow/shared/tauri-bridge/cmd/agent";
import { ViewShell } from "../shared/console-brand";
import { Button } from "@supportflow/ui/button";
import { Badge } from "@supportflow/ui/badge";
import type { AgentConsoleState } from "@supportflow/shared/contracts";

interface SkillsViewProps {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
}

export function SkillsView({ state, onRefresh }: SkillsViewProps) {
  const { t } = useTranslation("console");

  const handleRefresh = async () => {
    try {
      const skills = await refreshAgentSkills();
      if (state) {
        onRefresh({ ...state, skills });
      }
    } catch {
      // invokeWrapper throws InvokeError; state unchanged
    }
  };

  return (
    <ViewShell title={t("skills_title")} description={t("skills_desc")}>
      <div className="mb-4 flex justify-end">
        <Button type="button" variant="outline" size="sm" onClick={() => void handleRefresh()}>
          <RefreshCw className="mr-2 size-3.5" />
          {t("refresh")}
        </Button>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <section>
          <h3 className="mb-3 text-sm font-semibold">{t("skills_section_title")}</h3>
          <ul className="space-y-2">
            {(state?.skills ?? []).length === 0 ? (
              <li className="text-muted-foreground text-sm">{t("skills_loading_desc")}</li>
            ) : (
              state?.skills.map((skill) => (
                <li
                  key={skill.name}
                  className="rounded-lg border border-slate-200 px-4 py-3 dark:border-white/10"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium">{skill.name}</span>
                    <Badge variant={skill.enabled ? "default" : "secondary"}>
                      {skill.enabled ? t("skill_enabled") : t("skill_disabled")}
                    </Badge>
                  </div>
                  <p className="text-muted-foreground mt-1 text-sm">{skill.description}</p>
                  <p className="text-muted-foreground mt-1 font-mono text-xs">{skill.source}</p>
                </li>
              ))
            )}
          </ul>
        </section>

        <section>
          <h3 className="mb-3 text-sm font-semibold">{t("tools_section_title")}</h3>
          <ul className="space-y-2">
            {(state?.tools ?? []).map((tool) => (
              <li
                key={tool.name}
                className="rounded-lg border border-slate-200 px-4 py-3 dark:border-white/10"
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="font-medium">{tool.name}</span>
                  {tool.isMcp ? <Badge variant="outline">MCP</Badge> : null}
                </div>
                <p className="text-muted-foreground mt-1 text-sm">{tool.description}</p>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </ViewShell>
  );
}
