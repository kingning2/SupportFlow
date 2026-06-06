"use client";

import { useState } from "react";
import { Download, Eye, FileCode2, FolderTree, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  getAgentSkillDetail,
  installAgentSkill,
  refreshAgentSkills
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { ViewShell } from "../shared/console-brand";
import { Button } from "@supportflow/ui/button";
import { Badge } from "@supportflow/ui/badge";
import { Input } from "@supportflow/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle
} from "@supportflow/ui/dialog";
import type { AgentConsoleState, SkillDetail } from "@supportflow/shared/contracts";

interface SkillsViewProps {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
}

export function SkillsView({ state, onRefresh }: SkillsViewProps) {
  const { t } = useTranslation("console");
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [selectedSkill, setSelectedSkill] = useState<SkillDetail | null>(null);
  const [installSource, setInstallSource] = useState("");
  const [installing, setInstalling] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);
  const [installSuccess, setInstallSuccess] = useState<string | null>(null);

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

  const detailLabel = t("skills_view_detail", { defaultValue: "View details" });
  const detailTitle = t("skills_detail_title", { defaultValue: "Skill details" });
  const detailDesc = t("skills_detail_desc", {
    defaultValue: "Inspect where this skill comes from and how it is registered."
  });
  const detailLoadingText = t("skills_detail_loading", {
    defaultValue: "Loading skill details..."
  });
  const baseDirLabel = t("skills_detail_base_dir", { defaultValue: "Base directory" });
  const filePathLabel = t("skills_detail_file_path", { defaultValue: "Skill file" });
  const sourceLabel = t("skills_detail_source", { defaultValue: "Source" });
  const modelDisabledLabel = t("skills_detail_model_disabled", {
    defaultValue: "Model invocation disabled"
  });
  const modelEnabledLabel = t("skills_detail_model_enabled", {
    defaultValue: "Model invocation allowed"
  });
  const installTitle = t("skills_install_title", { defaultValue: "Install external skill" });
  const installDesc = t("skills_install_desc", {
    defaultValue: "Supports Skill Hub names, GitHub owner/repo, .zip URLs, and local paths."
  });
  const installPlaceholder = t("skills_install_placeholder", {
    defaultValue: "e.g. supportflow/notion-skill or https://example.com/skill.zip"
  });
  const installButtonLabel = t("skills_install_action", { defaultValue: "Install" });
  const installSuccessLabel = t("skills_install_success", {
    defaultValue: "Installed: {{names}}"
  });

  const handleOpenDetail = async (name: string) => {
    setDetailOpen(true);
    setDetailLoading(true);
    setDetailError(null);

    try {
      const detail = await getAgentSkillDetail(name);
      setSelectedSkill(detail);
    } catch (error) {
      setSelectedSkill(null);
      setDetailError(error instanceof Error ? error.message : String(error));
    } finally {
      setDetailLoading(false);
    }
  };

  const handleInstall = async () => {
    const source = installSource.trim();
    if (!source) {
      return;
    }

    setInstalling(true);
    setInstallError(null);
    setInstallSuccess(null);
    try {
      const result = await installAgentSkill({ source });
      const skills = await refreshAgentSkills();
      if (state) {
        onRefresh({ ...state, skills });
      }
      setInstallSuccess(
        installSuccessLabel.replace("{{names}}", result.installedNames.join(", ") || source)
      );
      setInstallSource("");
    } catch (error) {
      setInstallError(error instanceof Error ? error.message : String(error));
    } finally {
      setInstalling(false);
    }
  };

  return (
    <ViewShell title={t("skills_title")} description={t("skills_desc")}>
      <section className="mb-6 rounded-xl border border-slate-200 p-4 dark:border-white/10">
        <div className="mb-3">
          <h3 className="text-sm font-semibold">{installTitle}</h3>
          <p className="text-muted-foreground mt-1 text-sm">{installDesc}</p>
        </div>
        <div className="flex flex-col gap-3 sm:flex-row">
          <Input
            value={installSource}
            onChange={(event) => setInstallSource(event.target.value)}
            placeholder={installPlaceholder}
            className="font-mono text-sm"
          />
          <Button
            type="button"
            onClick={() => void handleInstall()}
            disabled={installing || installSource.trim().length === 0}
          >
            <Download className="mr-2 size-4" />
            {installButtonLabel}
          </Button>
        </div>
        {installError ? <p className="mt-3 text-sm text-red-500">{installError}</p> : null}
        {installSuccess ? (
          <p className="mt-3 text-sm text-emerald-600 dark:text-emerald-400">{installSuccess}</p>
        ) : null}
      </section>

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
                  <div className="mt-2 flex items-center justify-between gap-3">
                    <p className="text-muted-foreground min-w-0 flex-1 truncate font-mono text-xs">
                      {skill.source}
                    </p>
                    <Button
                      type="button"
                      size="sm"
                      variant="ghost"
                      onClick={() => void handleOpenDetail(skill.name)}
                    >
                      <Eye className="mr-1.5 size-3.5" />
                      {detailLabel}
                    </Button>
                  </div>
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

      <Dialog
        open={detailOpen}
        onOpenChange={(open) => {
          setDetailOpen(open);
          if (!open) {
            setSelectedSkill(null);
            setDetailError(null);
          }
        }}
      >
        <DialogContent className="sm:max-w-2xl">
          <DialogHeader>
            <DialogTitle>{selectedSkill?.name ?? detailTitle}</DialogTitle>
            <DialogDescription>{selectedSkill?.description ?? detailDesc}</DialogDescription>
          </DialogHeader>

          {detailLoading ? (
            <div className="text-muted-foreground py-6 text-sm">{detailLoadingText}</div>
          ) : detailError ? (
            <div className="py-6 text-sm text-red-500">{detailError}</div>
          ) : selectedSkill ? (
            <div className="space-y-4">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant={selectedSkill.enabled ? "default" : "secondary"}>
                  {selectedSkill.enabled ? t("skill_enabled") : t("skill_disabled")}
                </Badge>
                <Badge variant="outline">
                  {selectedSkill.disableModelInvocation ? modelDisabledLabel : modelEnabledLabel}
                </Badge>
              </div>

              <div className="grid gap-3">
                <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                  <div className="mb-1 flex items-center gap-2 text-sm font-medium">
                    <FolderTree className="size-4" />
                    {baseDirLabel}
                  </div>
                  <p className="text-muted-foreground font-mono text-xs break-all">
                    {selectedSkill.baseDir}
                  </p>
                </div>

                <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                  <div className="mb-1 flex items-center gap-2 text-sm font-medium">
                    <FileCode2 className="size-4" />
                    {filePathLabel}
                  </div>
                  <p className="text-muted-foreground font-mono text-xs break-all">
                    {selectedSkill.filePath}
                  </p>
                </div>

                <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                  <div className="mb-1 text-sm font-medium">{sourceLabel}</div>
                  <p className="text-muted-foreground font-mono text-xs">{selectedSkill.source}</p>
                </div>
              </div>
            </div>
          ) : null}
        </DialogContent>
      </Dialog>
    </ViewShell>
  );
}
