"use client";

import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
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

import { providerLabelKey } from "../lib/agent-console/provider-labels";
import { ViewShell } from "../shared/console-brand";
import { VendorCredentialDialog } from "../views/vendor-credential-dialog";

interface ModelsViewProps {
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
  t,
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
  t: ReturnType<typeof useTranslation>["t"];
  useCustomModel: boolean;
}) {
  return (
    <section className="mb-6 rounded-xl border border-slate-200 p-4 dark:border-white/10">
      <h3 className="mb-4 text-sm font-semibold">{t("models_active_title")}</h3>
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <span className="text-sm font-medium">{t("models_active_provider")}</span>
          <Select value={resolvedChatProviderId} onValueChange={onProviderChange}>
            <SelectTrigger>
              <SelectValue placeholder={t("models_select_provider")} />
            </SelectTrigger>
            <SelectContent>
              {configuredEditable.map((provider) => (
                <SelectItem key={provider.id} value={provider.id}>
                  {t(providerLabelKey(provider.id))}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <span className="text-sm font-medium">{t("models_active_model")}</span>
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
                <SelectItem value="__custom__">{t("models_custom_model")}</SelectItem>
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
              placeholder={t("models_custom_model")}
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
          {t("models_apply_chat")}
        </Button>
      </div>
    </section>
  );
}

function VendorsSection({
  details,
  onAdd,
  onEdit,
  t
}: {
  details: ModelProviderDetail[];
  onAdd: () => void;
  onEdit: (provider: ModelProviderDetail) => void;
  t: ReturnType<typeof useTranslation>["t"];
}) {
  return (
    <section>
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <h3 className="text-sm font-semibold">{t("models_section_vendors")}</h3>
        <Button type="button" size="sm" variant="outline" onClick={onAdd}>
          <Plus className="mr-1 h-4 w-4" />
          {t("models_add_vendor")}
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
              <span className="font-medium">{t(providerLabelKey(provider.id))}</span>
              {!provider.editable ? (
                <span className="text-muted-foreground text-xs">{t("models_readonly_vendor")}</span>
              ) : null}
            </div>
            <div className="flex items-center gap-2">
              <Badge variant={provider.configured ? "default" : "secondary"}>
                {provider.configured ? t("models_configured") : t("models_not_configured")}
              </Badge>
              {provider.isActive ? (
                <Badge className="bg-[#35A85B] text-white">{t("models_in_use")}</Badge>
              ) : null}
              {provider.editable ? (
                <Button
                  type="button"
                  size="icon"
                  variant="ghost"
                  className="h-8 w-8"
                  onClick={() => onEdit(provider)}
                  aria-label={t("models_edit_vendor")}
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

export function ModelsView({ state, onRefresh }: ModelsViewProps) {
  const { t } = useTranslation("console");
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
    <ViewShell title={t("models_title")} description={t("models_desc")}>
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
        t={t}
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
        t={t}
      />

      <p className="text-muted-foreground mt-6 text-xs">{t("models_edit_hint")}</p>

      <VendorCredentialDialog
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
