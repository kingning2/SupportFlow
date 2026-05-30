"use client";

import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { KeyRound, Pencil, Plus } from "lucide-react";

import { getAgentConsoleState, setAgentChatModel } from "@/cmd/agent";
import { ViewShell } from "@/components/agent-console/shared/console-brand";
import { VendorCredentialDialog } from "@/components/agent-console/views/vendor-credential-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import type { AgentConsoleState, ModelProviderDetail } from "@/generated/contracts";
import { providerLabelKey } from "@/lib/agent-console/provider-labels";

export function ModelsView({
  state,
  onRefresh
}: {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
}) {
  const { t } = useTranslation("console");
  const details = state?.providerDetails ?? [];

  const editableProviders = useMemo(() => details.filter((p) => p.editable), [details]);

  const configuredEditable = useMemo(
    () => editableProviders.filter((p) => p.configured),
    [editableProviders]
  );

  const activeDetail = useMemo(
    () => details.find((p) => p.isActive) ?? editableProviders[0],
    [details, editableProviders]
  );

  const [chatProviderId, setChatProviderId] = useState("");
  const [chatModel, setChatModel] = useState("");
  const [customModel, setCustomModel] = useState("");
  const [chatSaving, setChatSaving] = useState(false);

  const [vendorOpen, setVendorOpen] = useState(false);
  const [editingProvider, setEditingProvider] = useState<ModelProviderDetail | null>(null);

  const derivedChatProviderId = useMemo(() => {
    if (!state) {
      return "";
    }
    const match =
      editableProviders.find((p) => p.isActive) ??
      editableProviders.find((p) => p.botTypeValue === state.botType || p.id === state.botType);
    return match?.id ?? state.botType;
  }, [state, editableProviders]);

  const resolvedChatProviderId = chatProviderId || derivedChatProviderId || activeDetail?.id || "";
  const chatProvider =
    editableProviders.find((p) => p.id === resolvedChatProviderId) ?? activeDetail;

  const modelOptions = chatProvider?.models ?? [];
  const resolvedChatModel = chatModel || state?.modelName || modelOptions[0] || "";
  const resolvedCustomModel = customModel || state?.modelName || "";
  const useCustomModel =
    resolvedChatModel === "__custom__" ||
    (resolvedChatModel === "" && modelOptions.length === 0) ||
    (modelOptions.length > 0 && !modelOptions.includes(resolvedChatModel));

  const openEdit = (provider: ModelProviderDetail) => {
    setEditingProvider(provider);
    setVendorOpen(true);
  };

  const openAdd = () => {
    setEditingProvider(null);
    setVendorOpen(true);
  };

  const reloadState = async () => {
    const next = await getAgentConsoleState();
    onRefresh(next);
    if (!next) {
      return;
    }
    const editable = next.providerDetails.filter((p) => p.editable);
    const match =
      editable.find((p) => p.isActive) ??
      editable.find((p) => p.botTypeValue === next.botType || p.id === next.botType);
    setChatProviderId(match?.id ?? next.botType);
    setChatModel(next.modelName);
    setCustomModel(next.modelName);
  };

  const handleApplyChat = async () => {
    const modelValue = useCustomModel ? resolvedCustomModel.trim() : resolvedChatModel.trim();
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
      <section className="mb-6 rounded-xl border border-slate-200 p-4 dark:border-white/10">
        <h3 className="mb-4 text-sm font-semibold">{t("models_active_title")}</h3>
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <span className="text-sm font-medium">{t("models_active_provider")}</span>
            <Select
              value={resolvedChatProviderId}
              onValueChange={(id) => {
                setChatProviderId(id);
                const picked = editableProviders.find((p) => p.id === id);
                const firstModel = picked?.models[0];
                if (firstModel) {
                  setChatModel(firstModel);
                  setCustomModel(firstModel);
                }
              }}
            >
              <SelectTrigger>
                <SelectValue placeholder={t("models_select_provider")} />
              </SelectTrigger>
              <SelectContent>
                {configuredEditable.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    {t(providerLabelKey(p.id))}
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
                onValueChange={(v) => {
                  setChatModel(v);
                  if (v !== "__custom__") {
                    setCustomModel(v);
                  }
                }}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {modelOptions.map((m) => (
                    <SelectItem key={m} value={m}>
                      {m}
                    </SelectItem>
                  ))}
                  <SelectItem value="__custom__">{t("models_custom_model")}</SelectItem>
                </SelectContent>
              </Select>
            ) : (
              <Input
                className="font-mono text-sm"
                value={resolvedCustomModel}
                onChange={(e) => {
                  setCustomModel(e.target.value);
                  setChatModel(e.target.value);
                }}
              />
            )}
            {useCustomModel && modelOptions.length > 0 ? (
              <Input
                className="font-mono text-sm"
                placeholder={t("models_custom_model")}
                value={resolvedCustomModel}
                onChange={(e) => setCustomModel(e.target.value)}
              />
            ) : null}
          </div>
        </div>
        <div className="mt-4 flex flex-wrap items-center gap-3 text-sm">
          <span className="text-muted-foreground font-mono">
            bot_type: {state?.botType ?? "—"} · {state?.modelName ?? "—"}
          </span>
          <Button
            type="button"
            size="sm"
            disabled={chatSaving || configuredEditable.length === 0}
            onClick={() => void handleApplyChat()}
          >
            {t("models_apply_chat")}
          </Button>
        </div>
      </section>

      <section>
        <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
          <h3 className="text-sm font-semibold">{t("models_section_vendors")}</h3>
          <Button type="button" size="sm" variant="outline" onClick={openAdd}>
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
                  <span className="text-muted-foreground text-xs">
                    {t("models_readonly_vendor")}
                  </span>
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
                    onClick={() => openEdit(provider)}
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
