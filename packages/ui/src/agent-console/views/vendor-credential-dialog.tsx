"use client";

import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  clearAgentProvider,
  updateAgentProvider
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { Button } from "@supportflow/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from "@supportflow/ui/dialog";
import { Input } from "@supportflow/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@supportflow/ui/select";
import type { ModelProviderDetail } from "@supportflow/shared/contracts";
import { providerLabelKey } from "../lib/agent-console/provider-labels";

type VendorCredentialDialogProps = {
  open: boolean;
  provider: ModelProviderDetail | null;
  pickableProviders: ModelProviderDetail[];
  onOpenChange: (open: boolean) => void;
  onSaved: () => void | Promise<void>;
};

export function VendorCredentialDialog({
  open,
  provider,
  pickableProviders,
  onOpenChange,
  onSaved
}: VendorCredentialDialogProps) {
  const { t } = useTranslation("console");
  const [selectedId, setSelectedId] = useState(provider?.id ?? "");
  const [apiKey, setApiKey] = useState("");
  const [apiBaseDraft, setApiBaseDraft] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const active = provider ?? pickableProviders.find((p) => p.id === selectedId) ?? null;
  const resolvedApiBase = useMemo(() => {
    if (!active) {
      return "";
    }
    return apiBaseDraft || active.apiBase || active.apiBaseDefault || "";
  }, [active, apiBaseDraft]);

  const isPickerMode = !provider;

  const handleSave = async () => {
    if (!active) {
      return;
    }
    const trimmedKey = apiKey.trim();
    const hasMasked = Boolean(active.apiKeyMasked);
    if (!trimmedKey && !hasMasked) {
      setError(t("models_save_failed"));
      return;
    }

    setSaving(true);
    setError(null);
    try {
      await updateAgentProvider({
        providerId: active.id,
        apiKey: trimmedKey || undefined,
        apiBase: active.hasApiBase ? resolvedApiBase.trim() : undefined,
        apiBaseSet: active.hasApiBase
      });
      onOpenChange(false);
      await onSaved();
    } catch (err) {
      setError(err instanceof Error ? err.message : t("models_save_failed"));
    } finally {
      setSaving(false);
    }
  };

  const handleClear = async () => {
    if (!active) {
      return;
    }
    if (!window.confirm(`${t("models_clear_confirm_title")}\n${t("models_clear_confirm_msg")}`)) {
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await clearAgentProvider({ providerId: active.id });
      onOpenChange(false);
      await onSaved();
    } catch (err) {
      setError(err instanceof Error ? err.message : t("models_save_failed"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{provider ? t("models_edit_vendor") : t("models_add_vendor")}</DialogTitle>
          <DialogDescription className="font-mono text-xs">{active?.id ?? "—"}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {isPickerMode ? (
            <div className="space-y-2">
              <span className="text-sm font-medium">{t("models_select_provider")}</span>
              <Select value={selectedId} onValueChange={setSelectedId}>
                <SelectTrigger>
                  <SelectValue placeholder={t("models_select_provider")} />
                </SelectTrigger>
                <SelectContent>
                  {pickableProviders.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {t(providerLabelKey(p.id))}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ) : null}

          <div className="space-y-2">
            <label htmlFor="vendor-api-key" className="text-sm font-medium">
              {t("models_api_key")}
            </label>
            <Input
              id="vendor-api-key"
              type="password"
              autoComplete="off"
              placeholder={
                active?.apiKeyMasked ? active.apiKeyMasked : t("models_api_key_placeholder")
              }
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
            />
          </div>

          {active?.hasApiBase ? (
            <div className="space-y-2">
              <label htmlFor="vendor-api-base" className="text-sm font-medium">
                {t("models_api_base")}
              </label>
              <Input
                id="vendor-api-base"
                className="font-mono text-sm"
                placeholder={active.apiBaseDefault ?? ""}
                value={resolvedApiBase}
                onChange={(e) => setApiBaseDraft(e.target.value)}
              />
            </div>
          ) : null}

          {error ? <p className="text-sm text-red-500">{error}</p> : null}
        </div>

        <DialogFooter className="flex-col gap-2 sm:flex-row sm:justify-between">
          <Button
            type="button"
            variant="destructive"
            disabled={saving || !active?.configured}
            onClick={() => void handleClear()}
          >
            {t("models_clear_credential")}
          </Button>
          <div className="flex gap-2">
            <Button
              type="button"
              variant="outline"
              disabled={saving}
              onClick={() => onOpenChange(false)}
            >
              {t("models_cancel")}
            </Button>
            <Button type="button" disabled={saving || !active} onClick={() => void handleSave()}>
              {t("models_save")}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
