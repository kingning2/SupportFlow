"use client";

import { useMemo, useState } from "react";

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
import { providerLabel } from "../lib/agent-console/provider-labels";

type VendorCredentialsProps = {
  open: boolean;
  provider: ModelProviderDetail | null;
  pickableProviders: ModelProviderDetail[];
  onOpenChange: (open: boolean) => void;
  onSaved: () => void | Promise<void>;
};

export function VendorCredentials({
  open,
  provider,
  pickableProviders,
  onOpenChange,
  onSaved
}: VendorCredentialsProps) {
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
      setError("保存失败，请检查 Key 或网络权限。");
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
      setError(err instanceof Error ? err.message : "保存失败，请检查 Key 或网络权限。");
    } finally {
      setSaving(false);
    }
  };

  const handleClear = async () => {
    if (!active) {
      return;
    }
    if (!window.confirm("清除厂商凭据？\n这会删除当前厂商的 API Key 和 Base URL。")) {
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await clearAgentProvider({ providerId: active.id });
      onOpenChange(false);
      await onSaved();
    } catch (err) {
      setError(err instanceof Error ? err.message : "清除失败，请稍后再试。");
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{provider ? "编辑凭据" : "添加厂商"}</DialogTitle>
          <DialogDescription className="font-mono text-xs">{active?.id ?? "-"}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {isPickerMode ? (
            <div className="space-y-2">
              <span className="text-sm font-medium">选择厂商</span>
              <Select value={selectedId} onValueChange={setSelectedId}>
                <SelectTrigger>
                  <SelectValue placeholder="选择厂商" />
                </SelectTrigger>
                <SelectContent>
                  {pickableProviders.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {providerLabel(p.id)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ) : null}

          <div className="space-y-2">
            <label htmlFor="vendor-api-key" className="text-sm font-medium">
              API Key
            </label>
            <Input
              id="vendor-api-key"
              type="password"
              autoComplete="off"
              placeholder={active?.apiKeyMasked ? active.apiKeyMasked : "留空表示不修改已有 Key"}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
            />
          </div>

          {active?.hasApiBase ? (
            <div className="space-y-2">
              <label htmlFor="vendor-api-base" className="text-sm font-medium">
                API Base
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
            清除凭据
          </Button>
          <div className="flex gap-2">
            <Button
              type="button"
              variant="outline"
              disabled={saving}
              onClick={() => onOpenChange(false)}
            >
              取消
            </Button>
            <Button type="button" disabled={saving || !active} onClick={() => void handleSave()}>
              保存
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
