"use client";

import { useMemo, useState } from "react";
import { KeyRound, Pencil, Plus } from "lucide-react";

import {
  getAgentConsoleState,
  setAgentChatModel
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import type { AgentConsoleState, ModelProviderDetail } from "@supportflow/shared/contracts";
import { Badge } from "@supportflow/ui/badge";
import { Button } from "@supportflow/ui/button";
import { Input } from "@supportflow/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@supportflow/ui/select";

import { providerLabel } from "../lib/agent-console/provider-labels";
import { ViewShell } from "../shared/console-brand";
import { VendorCredentials } from "../views/vendor-credentials";

interface ModelsProps {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
}

interface ModelSelectionState {
  chatProviderId: string;
  chatModel: string;
  customModel: string;
  chatSaving: boolean;
}

function resolveProviderMatch(
  providers: ModelProviderDetail[],
  botType: string
): ModelProviderDetail | undefined {
  return (
    providers.find((provider) => provider.isActive) ??
    providers.find((provider) => provider.botTypeValue === botType || provider.id === botType)
  );
}

function shouldUseCustomModel(modelOptions: string[], resolvedChatModel: string) {
  return (
    resolvedChatModel === "__custom__" ||
    (resolvedChatModel === "" && modelOptions.length === 0) ||
    (modelOptions.length > 0 && !modelOptions.includes(resolvedChatModel))
  );
}

function resolveChatState(params: {
  activeDetail: ModelProviderDetail | undefined;
  chatProviderId: string;
  details: ModelProviderDetail[];
  state: AgentConsoleState | null;
}) {
  const { activeDetail, chatProviderId, details, state } = params;
  const editableProviders = details.filter((provider) => provider.editable);
  const configuredEditable = editableProviders.filter((provider) => provider.configured);
  const derivedChatProviderId = state
    ? (resolveProviderMatch(editableProviders, state.botType)?.id ?? state.botType)
    : "";
  const resolvedChatProviderId = chatProviderId || derivedChatProviderId || activeDetail?.id || "";
  const chatProvider =
    editableProviders.find((provider) => provider.id === resolvedChatProviderId) ?? activeDetail;

  return {
    chatProvider,
    configuredEditable,
    editableProviders,
    resolvedChatProviderId
  };
}

function ActiveModelSection({
  configuredEditable,
  modelSelection,
  modelOptions,
  onApply,
  onCustomModelChange,
  onModelChange,
  onProviderChange,
  resolvedChatModel,
  resolvedChatProviderId,
  resolvedCustomModel,
  state,
  useCustomModel
}: {
  configuredEditable: ModelProviderDetail[];
  modelOptions: string[];
  modelSelection: ModelSelectionState;
  onApply: () => Promise<void>;
  onCustomModelChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onProviderChange: (id: string) => void;
  resolvedChatModel: string;
  resolvedChatProviderId: string;
  resolvedCustomModel: string;
  state: AgentConsoleState | null;
  useCustomModel: boolean;
}) {
  return (
    <section className="mb-6 rounded-xl border border-slate-200 p-4 dark:border-white/10">
      <h3 className="mb-4 text-sm font-semibold">当前对话模型</h3>
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <span className="text-sm font-medium">厂商</span>
          <Select value={resolvedChatProviderId} onValueChange={onProviderChange}>
            <SelectTrigger>
              <SelectValue placeholder="选择厂商" />
            </SelectTrigger>
            <SelectContent>
              {configuredEditable.map((provider) => (
                <SelectItem key={provider.id} value={provider.id}>
                  {providerLabel(provider.id)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <span className="text-sm font-medium">模型 ID</span>
          {modelOptions.length > 0 ? (
            <Select
              value={useCustomModel ? "__custom__" : resolvedChatModel}
              onValueChange={onModelChange}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {modelOptions.map((model) => (
                  <SelectItem key={model} value={model}>
                    {model}
                  </SelectItem>
                ))}
                <SelectItem value="__custom__">自定义模型名</SelectItem>
              </SelectContent>
            </Select>
          ) : (
            <Input
              className="font-mono text-sm"
              value={resolvedCustomModel}
              onChange={(event) => onCustomModelChange(event.target.value)}
            />
          )}

          {useCustomModel && modelOptions.length > 0 ? (
            <Input
              className="font-mono text-sm"
              placeholder="自定义模型名"
              value={resolvedCustomModel}
              onChange={(event) => onCustomModelChange(event.target.value)}
            />
          ) : null}
        </div>
      </div>

      <div className="mt-4 flex flex-wrap items-center gap-3 text-sm">
        <span className="text-muted-foreground font-mono">
          bot_type: {state?.botType ?? "N/A"} | {state?.modelName ?? "N/A"}
        </span>
        <Button
          type="button"
          size="sm"
          disabled={modelSelection.chatSaving || configuredEditable.length === 0}
          onClick={() => void onApply()}
        >
          应用对话模型
        </Button>
      </div>
    </section>
  );
}

function VendorsSection({
  details,
  onAdd,
  onEdit
}: {
  details: ModelProviderDetail[];
  onAdd: () => void;
  onEdit: (provider: ModelProviderDetail) => void;
}) {
  return (
    <section>
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <h3 className="text-sm font-semibold">厂商凭据</h3>
        <Button type="button" size="sm" variant="outline" onClick={onAdd}>
          <Plus className="mr-1 h-4 w-4" />
          添加厂商
        </Button>
      </div>
      <ul className="space-y-2">
        {details.map((provider) => (
          <li
            key={provider.id}
            className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-slate-200 px-4 py-3 dark:border-white/10"
          >
            <div className="flex min-w-0 items-center gap-2">
              <KeyRound className="text-muted-foreground h-4 w-4 shrink-0" />
              <span className="font-medium">{providerLabel(provider.id)}</span>
              {!provider.editable ? (
                <span className="text-muted-foreground text-xs">当前厂商暂不支持在控制台配置</span>
              ) : null}
            </div>
            <div className="flex items-center gap-2">
              <Badge variant={provider.configured ? "default" : "secondary"}>
                {provider.configured ? "已配置 API Key" : "未配置"}
              </Badge>
              {provider.isActive ? <Badge className="bg-[#35A85B] text-white">使用中</Badge> : null}
              {provider.editable ? (
                <Button
                  type="button"
                  size="icon"
                  variant="ghost"
                  className="h-8 w-8"
                  onClick={() => onEdit(provider)}
                  aria-label="编辑凭据"
                >
                  <Pencil className="h-4 w-4" />
                </Button>
              ) : null}
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function Models({ state, onRefresh }: ModelsProps) {
  const details = state?.providerDetails ?? [];
  const activeDetail = useMemo(
    () =>
      details.find((provider) => provider.isActive) ??
      details.find((provider) => provider.editable),
    [details]
  );

  const [chatProviderId, setChatProviderId] = useState("");
  const [chatModel, setChatModel] = useState("");
  const [customModel, setCustomModel] = useState("");
  const [chatSaving, setChatSaving] = useState(false);
  const [vendorOpen, setVendorOpen] = useState(false);
  const [editingProvider, setEditingProvider] = useState<ModelProviderDetail | null>(null);

  const { chatProvider, configuredEditable, editableProviders, resolvedChatProviderId } =
    resolveChatState({
      activeDetail,
      chatProviderId,
      details,
      state
    });
  const modelOptions = chatProvider?.models ?? [];
  const resolvedChatModel = chatModel || state?.modelName || modelOptions[0] || "";
  const resolvedCustomModel = customModel || state?.modelName || "";
  const useCustomModel = shouldUseCustomModel(modelOptions, resolvedChatModel);

  const reloadState = async () => {
    const next = await getAgentConsoleState();
    onRefresh(next);
    if (!next) {
      return;
    }
    const editable = next.providerDetails.filter((provider) => provider.editable);
    const match = resolveProviderMatch(editable, next.botType);
    setChatProviderId(match?.id ?? next.botType);
    setChatModel(next.modelName);
    setCustomModel(next.modelName);
  };

  const handleProviderChange = (id: string) => {
    setChatProviderId(id);
    const picked = editableProviders.find((provider) => provider.id === id);
    const firstModel = picked?.models[0] ?? "";
    setChatModel(firstModel);
    setCustomModel(firstModel);
  };

  const handleModelChange = (value: string) => {
    setChatModel(value);
    if (value !== "__custom__") {
      setCustomModel(value);
    }
  };

  const handleCustomModelChange = (value: string) => {
    setCustomModel(value);
    if (modelOptions.length === 0) {
      setChatModel(value);
    }
  };

  const handleApplyChat = async () => {
    const modelValue = (useCustomModel ? resolvedCustomModel : resolvedChatModel).trim();
    if (!resolvedChatProviderId || !modelValue) {
      return;
    }
    setChatSaving(true);
    try {
      await setAgentChatModel({
        providerId: resolvedChatProviderId,
        model: modelValue
      });
      await reloadState();
    } finally {
      setChatSaving(false);
    }
  };

  return (
    <ViewShell title="模型管理" description="配置各厂商 API Key，并选择当前对话使用的模型。">
      <ActiveModelSection
        configuredEditable={configuredEditable}
        modelOptions={modelOptions}
        modelSelection={{ chatModel, chatProviderId, chatSaving, customModel }}
        onApply={handleApplyChat}
        onCustomModelChange={handleCustomModelChange}
        onModelChange={handleModelChange}
        onProviderChange={handleProviderChange}
        resolvedChatModel={resolvedChatModel}
        resolvedChatProviderId={resolvedChatProviderId}
        resolvedCustomModel={resolvedCustomModel}
        state={state}
        useCustomModel={useCustomModel}
      />

      <VendorsSection
        details={details}
        onAdd={() => {
          setEditingProvider(null);
          setVendorOpen(true);
        }}
        onEdit={(provider) => {
          setEditingProvider(provider);
          setVendorOpen(true);
        }}
      />

      <p className="text-muted-foreground mt-6 text-xs">
        凭据会写入 `src-tauri/resources/config.json`，保存后立即生效，无需重启。
      </p>

      <VendorCredentials
        key={`${editingProvider?.id ?? "new"}-${vendorOpen ? "open" : "closed"}`}
        open={vendorOpen}
        provider={editingProvider}
        pickableProviders={editableProviders}
        onOpenChange={setVendorOpen}
        onSaved={reloadState}
      />
    </ViewShell>
  );
}
