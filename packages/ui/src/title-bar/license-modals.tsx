"use client";

import { Modal } from "antd";
import { Copy } from "lucide-react";
import { useCallback, useState } from "react";

import {
  getLicenseStatus,
  pickAndApplyLicenseActivationKey,
  type LicenseStatusDto
} from "@supportflow/shared/tauri-bridge/cmd/license";
import { Button } from "@supportflow/ui/button";
import { Input } from "@supportflow/ui/input";

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
  const { status, error, setError, loadStatus, setStatus } = useLicenseStatus();
  const [activating, setActivating] = useState(false);

  const handleClose = useCallback(() => {
    onOpenChange(false);
    setError(null);
  }, [onOpenChange, setError]);

  const handleApplyActivation = useCallback(async () => {
    setActivating(true);
    setError(null);
    try {
      const next = await pickAndApplyLicenseActivationKey();
      setStatus(next);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setActivating(false);
    }
  }, [setError, setStatus]);

  const isLicensed = status?.valid === true;

  return (
    <Modal
      open={open}
      title={"订阅激活"}
      onCancel={handleClose}
      afterOpenChange={(visible) => {
        if (visible) void loadStatus();
      }}
      destroyOnHidden
      footer={[
        <Button key="cancel" type="button" variant="outline" onClick={handleClose}>
          {"取消"}
        </Button>,
        <Button
          key="apply"
          type="button"
          disabled={activating}
          onClick={() => void handleApplyActivation()}
        >
          {activating ? "激活中…" : "激活"}
        </Button>
      ]}
    >
      <p className="text-muted-foreground mb-3 text-sm">
        {isLicensed ? "已激活，可正常使用" : "粘贴管理员提供的激活码后点击激活"}
      </p>
      {error ? <p className="mt-2 text-sm text-red-600 dark:text-red-400">{error}</p> : null}
    </Modal>
  );
}

/** 机器码复制弹窗 */
export function LicenseMachineCodeModal({ open, onOpenChange }: LicenseModalProps) {
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
        setError("无法访问剪贴板");
        return;
      }
      await navigator.clipboard.writeText(machineCode);
      setMachineCopied(true);
      window.setTimeout(() => setMachineCopied(false), 2000);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    }
  }, [machineCode, setError]);

  return (
    <Modal
      open={open}
      title={"机器码"}
      onCancel={handleClose}
      afterOpenChange={(visible) => {
        if (visible) void loadStatus();
      }}
      destroyOnHidden
      footer={[
        <Button key="cancel" type="button" variant="outline" onClick={handleClose}>
          {"取消"}
        </Button>,
        <Button
          key="copy"
          type="button"
          disabled={!machineCode}
          onClick={() => void handleCopyMachineCode()}
        >
          <Copy className="mr-1.5 size-3.5" />
          {machineCopied ? "已复制" : "复制机器码"}
        </Button>
      ]}
    >
      <p className="text-muted-foreground mb-3 text-sm">{"复制机器码发给管理员"}</p>
      <Input className="font-mono text-xs" value={machineCode} readOnly />
      {error ? <p className="mt-2 text-sm text-red-600 dark:text-red-400">{error}</p> : null}
    </Modal>
  );
}
