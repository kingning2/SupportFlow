"use client";

import { useState } from "react";
import { Download, Eye, FileCode2, FolderTree, RefreshCw } from "lucide-react";

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

interface SkillsProps {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
}

const COPY = {
  baseDirLabel: "基础目录",
  detailDesc: "查看技能来源、注册位置和当前状态。",
  detailLabel: "查看详情",
  detailLoadingText: "正在加载技能详情...",
  detailTitle: "技能详情",
  filePathLabel: "技能文件",
  installButtonLabel: "安装",
  installDesc: "支持 Skill Hub 名称、GitHub owner/repo、zip 链接和本地路径。",
  installPlaceholder: "例如：supportflow/notion-skill 或 https://example.com/skill.zip",
  installTitle: "安装外部技能",
  installSuccessPrefix: "安装成功：",
  modelDisabledLabel: "已禁用模型调用",
  modelEnabledLabel: "允许模型调用",
  sourceLabel: "来源"
} as const;

function SkillInstallSection({
  installError,
  installing,
  installSource,
  installSuccess,
  onChange,
  onInstall
}: {
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
        <h3 className="text-sm font-semibold">{COPY.installTitle}</h3>
        <p className="text-muted-foreground mt-1 text-sm">{COPY.installDesc}</p>
      </div>
      <div className="flex flex-col gap-3 sm:flex-row">
        <Input
          value={installSource}
          onChange={(event) => onChange(event.target.value)}
          placeholder={COPY.installPlaceholder}
          className="font-mono text-sm"
        />
        <Button
          type="button"
          onClick={() => void onInstall()}
          disabled={installing || installSource.trim().length === 0}
        >
          <Download className="mr-2 size-4" />
          {COPY.installButtonLabel}
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
  skills
}: {
  detailLabel: string;
  onOpenDetail: (name: string) => Promise<void>;
  skills: AgentConsoleState["skills"];
}) {
  return (
    <section>
      <h3 className="mb-3 text-sm font-semibold">技能</h3>
      <ul className="space-y-2">
        {(skills ?? []).length === 0 ? (
          <li className="text-muted-foreground text-sm">
            暂无技能，可在工作区 `skills/` 目录下添加。
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

function ToolsListSection({ tools }: { tools: AgentConsoleState["tools"] }) {
  return (
    <section>
      <h3 className="mb-3 text-sm font-semibold">工具</h3>
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
  detailError,
  detailLoading,
  detailOpen,
  onOpenChange,
  selectedSkill
}: {
  detailError: string | null;
  detailLoading: boolean;
  detailOpen: boolean;
  onOpenChange: (open: boolean) => void;
  selectedSkill: SkillDetail | null;
}) {
  return (
    <Dialog open={detailOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{selectedSkill?.name ?? COPY.detailTitle}</DialogTitle>
          <DialogDescription>{selectedSkill?.description ?? COPY.detailDesc}</DialogDescription>
        </DialogHeader>

        {detailLoading ? (
          <div className="text-muted-foreground py-6 text-sm">{COPY.detailLoadingText}</div>
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
                  ? COPY.modelDisabledLabel
                  : COPY.modelEnabledLabel}
              </Badge>
            </div>

            <div className="grid gap-3">
              <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                <div className="mb-1 flex items-center gap-2 text-sm font-medium">
                  <FolderTree className="size-4" />
                  {COPY.baseDirLabel}
                </div>
                <p className="text-muted-foreground font-mono text-xs break-all">
                  {selectedSkill.baseDir}
                </p>
              </div>

              <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                <div className="mb-1 flex items-center gap-2 text-sm font-medium">
                  <FileCode2 className="size-4" />
                  {COPY.filePathLabel}
                </div>
                <p className="text-muted-foreground font-mono text-xs break-all">
                  {selectedSkill.filePath}
                </p>
              </div>

              <div className="rounded-lg border border-slate-200 p-3 dark:border-white/10">
                <div className="mb-1 text-sm font-medium">{COPY.sourceLabel}</div>
                <p className="text-muted-foreground font-mono text-xs">{selectedSkill.source}</p>
              </div>
            </div>
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}

export function Skills({ state, onRefresh }: SkillsProps) {
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
        `${COPY.installSuccessPrefix}${result.installed_names.join(", ") || source}`
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
      title="技能与工具"
      description="查看当前已注册的工具与技能，并支持安装新的外部技能。"
    >
      <SkillInstallSection
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
          刷新
        </Button>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <SkillsListSection
          detailLabel={COPY.detailLabel}
          onOpenDetail={handleOpenDetail}
          skills={state?.skills ?? []}
        />
        <ToolsListSection tools={state?.tools ?? []} />
      </div>

      <SkillDetailDialog
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
      />
    </ViewShell>
  );
}
