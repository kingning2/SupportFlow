"use client";

import { useMemo, useState } from "react";
import { Download, Eye, FileCode2, FolderTree, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  getAgentSkillDetail,
  installAgentSkill,
  refreshAgentSkills
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import type { AgentConsoleState, SkillDetail } from "@supportflow/shared/contracts";
import { Badge } from "@supportflow/ui/badge";
import { Button } from "@supportflow/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle
} from "@supportflow/ui/dialog";
import { Input } from "@supportflow/ui/input";

import { ViewShell } from "../shared/console-brand";

interface SkillsViewProps {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
}

interface SkillsCopy {
  detailDesc: string;
  detailLabel: string;
  detailLoadingText: string;
  detailTitle: string;
  filePathLabel: string;
  installButtonLabel: string;
  installDesc: string;
  installPlaceholder: string;
  installSuccessLabel: string;
  installTitle: string;
  modelDisabledLabel: string;
  modelEnabledLabel: string;
  sourceLabel: string;
  baseDirLabel: string;
}

function useSkillsCopy(t: ReturnType<typeof useTranslation>["t"]): SkillsCopy {
  return useMemo(
    () => ({
      baseDirLabel: t("skills_detail_base_dir", { defaultValue: "Base directory" }),
      detailDesc: t("skills_detail_desc", {
        defaultValue: "Inspect where this skill comes from and how it is registered."
      }),
      detailLabel: t("skills_view_detail", { defaultValue: "View details" }),
      detailLoadingText: t("skills_detail_loading", {
        defaultValue: "Loading skill details..."
      }),
      detailTitle: t("skills_detail_title", { defaultValue: "Skill details" }),
      filePathLabel: t("skills_detail_file_path", { defaultValue: "Skill file" }),
      installButtonLabel: t("skills_install_action", { defaultValue: "Install" }),
      installDesc: t("skills_install_desc", {
        defaultValue: "Supports Skill Hub names, GitHub owner/repo, .zip URLs, and local paths."
      }),
      installPlaceholder: t("skills_install_placeholder", {
        defaultValue: "e.g. supportflow/notion-skill or https://example.com/skill.zip"
      }),
      installSuccessLabel: t("skills_install_success", {
        defaultValue: "Installed: {{names}}"
      }),
      installTitle: t("skills_install_title", { defaultValue: "Install external skill" }),
      modelDisabledLabel: t("skills_detail_model_disabled", {
        defaultValue: "Model invocation disabled"
      }),
      modelEnabledLabel: t("skills_detail_model_enabled", {
        defaultValue: "Model invocation allowed"
      }),
      sourceLabel: t("skills_detail_source", { defaultValue: "Source" })
    }),
    [t]
  );
}

function SkillInstallSection({
  copy,
  installError,
  installing,
  installSource,
  installSuccess,
  onChange,
  onInstall
}: {
  copy: SkillsCopy;
  installError: string | null;
  installing: boolean;
  installSource: string;
  installSuccess: string | null;
  onChange: (value: string) => void;
  onInstall: () => Promise<void>;
}) {
  return (
    <section className="mb-6 rounded-xl border border-slate-200 p-4 dark:border-white/10">
      <div className="mb-3">
        <h3 className="text-sm font-semibold">{copy.installTitle}</h3>
        <p className="text-muted-foreground mt-1 text-sm">{copy.installDesc}</p>
      </div>
      <div className="flex flex-col gap-3 sm:flex-row">
        <Input
          value={installSource}
          onChange={(event) => onChange(event.target.value)}
          placeholder={copy.installPlaceholder}
          className="font-mono text-sm"
        />
        <Button
          type="button"
          onClick={() => void onInstall()}
          disabled={installing || installSource.trim().length === 0}
        >
          <Download className="mr-2 size-4" />
          {copy.installButtonLabel}
        </Button>
      </div>
      {installError ? <p className="mt-3 text-sm text-red-500">{installError}</p> : null}
      {installSuccess ? (
        <p className="mt-3 text-sm text-emerald-600 dark:text-emerald-400">{installSuccess}</p>
      ) : null}
    </section>
  );
}

function SkillsListSection({
  detailLabel,
  onOpenDetail,
  skills,
  t
}: {
  detailLabel: string;
  onOpenDetail: (name: string) => Promise<void>;
  skills: AgentConsoleState["skills"];
  t: ReturnType<typeof useTranslation>["t"];
}) {
  return (
    <section>
      <h3 className="mb-3 text-sm font-semibold">{"技能"}</h3>
      <ul className="space-y-2">
        {(skills ?? []).length === 0 ? (
          <li className="text-muted-foreground text-sm">
            {"暂无技能，可在工作区 skills/ 目录添加。"}
          </li>
        ) : (
          skills?.map((skill) => (
            <li
              key={skill.name}
              className="rounded-lg border border-slate-200 px-4 py-3 dark:border-white/10"
            >
              <div className="flex items-center justify-between gap-2">
                <span className="font-medium">{skill.name}</span>
                <Badge variant={skill.enabled ? "default" : "secondary"}>
                  {skill.enabled ? "已启用" : "已禁用"}
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
                  onClick={() => void onOpenDetail(skill.name)}
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
  );
}

function ToolsListSection({
  tools,
  t
}: {
  tools: AgentConsoleState["tools"];
  t: ReturnType<typeof useTranslation>["t"];
}) {
  return (
    <section>
      <h3 className="mb-3 text-sm font-semibold">{"工具"}</h3>
      <ul className="space-y-2">
        {(tools ?? []).map((tool) => (
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
  );
}

function SkillDetailDialog({
  copy,
  detailError,
  detailLoading,
  detailOpen,
  onOpenChange,
  selectedSkill,
  t
}: {
  copy: SkillsCopy;
  detailError: string | null;
  detailLoading: boolean;
  detailOpen: boolean;
  onOpenChange: (open: boolean) => void;
  selectedSkill: SkillDetail | null;
  t: ReturnType<typeof useTranslation>["t"];
}) {
  return (
    <Dialog open={detailOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{selectedSkill?.name ?? copy.detailTitle}</DialogTitle>
          <DialogDescription>{selectedSkill?.description ?? copy.detailDesc}</DialogDescription>
        </DialogHeader>

        {detailLoading ? (
          <div className="text-muted-foreground py-6 text-sm">{copy.detailLoadingText}</div>
        ) : null}
        {!detailLoading && detailError ? (
          <div className="py-6 text-sm text-red-500">{detailError}</div>
        ) : null}
        {!detailLoading && !detailError && selectedSkill ? (
          <div className="space-y-4">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={selectedSkill.enabled ? "default" : "secondary"}>
                {selectedSkill.enabled ? "已启用" : "已禁用"}
              </Badge>
              <Badge variant="outline">
                {selectedSkill.disableModelInvocation
                  ? copy.modelDisabledLabel
                  : copy.modelEnabledLabel}
              </Badge>
            </div>

            <div className="grid gap-3">
              <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                <div className="mb-1 flex items-center gap-2 text-sm font-medium">
                  <FolderTree className="size-4" />
                  {copy.baseDirLabel}
                </div>
                <p className="text-muted-foreground font-mono text-xs break-all">
                  {selectedSkill.baseDir}
                </p>
              </div>

              <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                <div className="mb-1 flex items-center gap-2 text-sm font-medium">
                  <FileCode2 className="size-4" />
                  {copy.filePathLabel}
                </div>
                <p className="text-muted-foreground font-mono text-xs break-all">
                  {selectedSkill.filePath}
                </p>
              </div>

              <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                <div className="mb-1 text-sm font-medium">{copy.sourceLabel}</div>
                <p className="text-muted-foreground font-mono text-xs">{selectedSkill.source}</p>
              </div>
            </div>
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}

export function SkillsView({ state, onRefresh }: SkillsViewProps) {
  const { t } = useTranslation("console");
  const copy = useSkillsCopy(t);
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [selectedSkill, setSelectedSkill] = useState<SkillDetail | null>(null);
  const [installSource, setInstallSource] = useState("");
  const [installing, setInstalling] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);
  const [installSuccess, setInstallSuccess] = useState<string | null>(null);

  const refreshSkillsState = async () => {
    try {
      const skills = await refreshAgentSkills();
      if (state) {
        onRefresh({ ...state, skills });
      }
    } catch {
      // keep current state
    }
  };

  const handleOpenDetail = async (name: string) => {
    setDetailOpen(true);
    setDetailLoading(true);
    setDetailError(null);

    try {
      setSelectedSkill(await getAgentSkillDetail(name));
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
      await refreshSkillsState();
      setInstallSuccess(
        copy.installSuccessLabel.replace("{{names}}", result.installed_names.join(", ") || source)
      );
      setInstallSource("");
    } catch (error) {
      setInstallError(error instanceof Error ? error.message : String(error));
    } finally {
      setInstalling(false);
    }
  };

  return (
    <ViewShell
      title={"技能与工具"}
      description={"当前进程内已注册的工具与技能（通过 Tauri invoke 读取）。"}
    >
      <SkillInstallSection
        copy={copy}
        installError={installError}
        installing={installing}
        installSource={installSource}
        installSuccess={installSuccess}
        onChange={setInstallSource}
        onInstall={handleInstall}
      />

      <div className="mb-4 flex justify-end">
        <Button type="button" variant="outline" size="sm" onClick={() => void refreshSkillsState()}>
          <RefreshCw className="mr-2 size-3.5" />
          {"刷新"}
        </Button>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <SkillsListSection
          detailLabel={copy.detailLabel}
          onOpenDetail={handleOpenDetail}
          skills={state?.skills ?? []}
          t={t}
        />
        <ToolsListSection tools={state?.tools ?? []} t={t} />
      </div>

      <SkillDetailDialog
        copy={copy}
        detailError={detailError}
        detailLoading={detailLoading}
        detailOpen={detailOpen}
        onOpenChange={(open) => {
          setDetailOpen(open);
          if (!open) {
            setSelectedSkill(null);
            setDetailError(null);
          }
        }}
        selectedSkill={selectedSkill}
        t={t}
      />
    </ViewShell>
  );
}
