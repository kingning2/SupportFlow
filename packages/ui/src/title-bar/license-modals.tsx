"use client";

import { Modal } from "antd";
import { Copy } from "lucide-react";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  applyLicenseActivation,
  getLicenseStatus,
  type LicenseStatusDto
} from "@supportflow/shared/tauri-bridge/cmd/license";
import { Button } from "@supportflow/ui/button";
import { Input } from "@supportflow/ui/input";
import { Textarea } from "@supportflow/ui/textarea";

type LicenseModalProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

function useLicenseStatus() {
  const [status, setStatus] = useState<LicenseStatusDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadStatus = useCallback(async () => {
    try {
      const next = await getLicenseStatus();
      setStatus(next);
      setError(null);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    }
  }, []);

  return { status, error, setError, loadStatus, setStatus };
}

/** 订阅激活弹窗 */
export function LicenseActivationModal({ open, onOpenChange }: LicenseModalProps) {
  const { t } = useTranslation("title_bar");
  const { status, error, setError, loadStatus, setStatus } = useLicenseStatus();
  const [activationToken, setActivationToken] = useState("");
  const [activating, setActivating] = useState(false);

  const handleClose = useCallback(() => {
    onOpenChange(false);
    setActivationToken("");
    setError(null);
  }, [onOpenChange, setError]);

  const handleApplyActivation = useCallback(async () => {
    const token = activationToken.trim();
    if (!token) {
      setError(t("license_activation_empty"));
      return;
    }
    setActivating(true);
    setError(null);
    try {
      const next = await applyLicenseActivation(token);
      setStatus(next);
      setActivationToken("");
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setActivating(false);
    }
  }, [activationToken, setError, setStatus, t]);

  const isLicensed = status?.valid === true;

  return (
    <Modal
      open={open}
      title={t("license_activation_title")}
      onCancel={handleClose}
      afterOpenChange={(visible) => {
        if (visible) void loadStatus();
      }}
      destroyOnHidden
      footer={[
        <Button key="cancel" type="button" variant="outline" onClick={handleClose}>
          {t("license_modal_cancel")}
        </Button>,
        <Button
          key="apply"
          type="button"
          disabled={activating}
          onClick={() => void handleApplyActivation()}
        >
          {activating ? t("license_activation_applying") : t("license_activation_apply")}
        </Button>
      ]}
    >
      <p className="text-muted-foreground mb-3 text-sm">
        {isLicensed ? t("license_activation_active") : t("license_activation_hint")}
      </p>
      <Textarea
        className="min-h-[96px] font-mono text-xs"
        value={activationToken}
        onChange={(e) => setActivationToken(e.target.value)}
        placeholder={t("license_activation_placeholder")}
      />
      {error ? <p className="mt-2 text-sm text-red-600 dark:text-red-400">{error}</p> : null}
    </Modal>
  );
}

/** 机器码复制弹窗 */
export function LicenseMachineCodeModal({ open, onOpenChange }: LicenseModalProps) {
  const { t } = useTranslation("title_bar");
  const { status, error, setError, loadStatus } = useLicenseStatus();
  const [machineCopied, setMachineCopied] = useState(false);

  const machineCode = status?.machineCode ?? "";

  const handleClose = useCallback(() => {
    onOpenChange(false);
    setMachineCopied(false);
    setError(null);
  }, [onOpenChange, setError]);

  const handleCopyMachineCode = useCallback(async () => {
    if (!machineCode) {
      return;
    }
    setMachineCopied(false);
    setError(null);
    try {
      if (typeof window === "undefined" || !navigator?.clipboard?.writeText) {
        setError(t("license_machine_code_clipboard_unavailable"));
        return;
      }
      await navigator.clipboard.writeText(machineCode);
      setMachineCopied(true);
      window.setTimeout(() => setMachineCopied(false), 2000);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    }
  }, [machineCode, setError, t]);

  return (
    <Modal
      open={open}
      title={t("license_machine_code_label")}
      onCancel={handleClose}
      afterOpenChange={(visible) => {
        if (visible) void loadStatus();
      }}
      destroyOnHidden
      footer={[
        <Button key="cancel" type="button" variant="outline" onClick={handleClose}>
          {t("license_modal_cancel")}
        </Button>,
        <Button
          key="copy"
          type="button"
          disabled={!machineCode}
          onClick={() => void handleCopyMachineCode()}
        >
          <Copy className="mr-1.5 size-3.5" />
          {machineCopied ? t("license_machine_code_copied") : t("license_machine_code_copy")}
        </Button>
      ]}
    >
      <p className="text-muted-foreground mb-3 text-sm">{t("license_machine_code_hint")}</p>
      <Input className="font-mono text-xs" value={machineCode} readOnly />
      {error ? <p className="mt-2 text-sm text-red-600 dark:text-red-400">{error}</p> : null}
    </Modal>
  );
}
