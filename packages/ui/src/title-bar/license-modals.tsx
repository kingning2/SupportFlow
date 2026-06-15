"use client";

import type { ReactNode } from "react";
import { IconCopy } from "@douyinfe/semi-icons";
import { Button, Input, Modal, Space, Typography } from "@douyinfe/semi-ui-19";
import { useCallback, useEffect, useState } from "react";

import {
  getLicenseStatus,
  pickAndApplyLicenseActivationKey,
  type LicenseStatusDto
} from "@supportflow/shared/tauri-bridge/cmd/license";
import { useOptionalLicenseGate } from "@supportflow/ui/license";

const { Text, Paragraph } = Typography;

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

function ModalFooter({ children }: { children: ReactNode }) {
  return <Space style={{ width: "100%", justifyContent: "flex-end" }}>{children}</Space>;
}

/** 订阅激活弹窗 */
export function LicenseActivationModal({ open, onOpenChange }: LicenseModalProps) {
  const { status, error, setError, loadStatus, setStatus } = useLicenseStatus();
  const licenseGate = useOptionalLicenseGate();
  const [activating, setActivating] = useState(false);

  useEffect(() => {
    if (open) {
      void loadStatus();
    }
  }, [loadStatus, open]);

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
      licenseGate?.applyStatus(next);
      if (next.valid) {
        onOpenChange(false);
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setActivating(false);
    }
  }, [licenseGate, onOpenChange, setError, setStatus]);

  const isLicensed = status?.valid === true;

  return (
    <Modal
      visible={open}
      title="订阅激活"
      onCancel={handleClose}
      footer={
        <ModalFooter>
          <Button theme="light" type="tertiary" onClick={handleClose}>
            取消
          </Button>
          <Button type="primary" loading={activating} onClick={() => void handleApplyActivation()}>
            {activating ? "激活中…" : "激活"}
          </Button>
        </ModalFooter>
      }
    >
      <Paragraph type="tertiary" style={{ marginBottom: 12 }}>
        {isLicensed ? "已激活，可正常使用" : "选择管理员提供的激活文件后点击激活"}
      </Paragraph>
      {error ? (
        <Text type="danger" style={{ display: "block", marginTop: 8 }}>
          {error}
        </Text>
      ) : null}
    </Modal>
  );
}

/** 机器码复制弹窗 */
export function LicenseMachineCodeModal({ open, onOpenChange }: LicenseModalProps) {
  const { status, error, setError, loadStatus } = useLicenseStatus();
  const [machineCopied, setMachineCopied] = useState(false);

  const machineCode = status?.machineCode ?? "";

  useEffect(() => {
    if (open) {
      void loadStatus();
    }
  }, [loadStatus, open]);

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
      visible={open}
      title="机器码"
      onCancel={handleClose}
      footer={
        <ModalFooter>
          <Button theme="light" type="tertiary" onClick={handleClose}>
            取消
          </Button>
          <Button
            type="primary"
            disabled={!machineCode}
            onClick={() => void handleCopyMachineCode()}
          >
            <IconCopy />
            {machineCopied ? "已复制" : "复制机器码"}
          </Button>
        </ModalFooter>
      }
    >
      <Paragraph type="tertiary" style={{ marginBottom: 12 }}>
        复制机器码发给管理员
      </Paragraph>
      <Input style={{ fontFamily: "monospace", fontSize: 12 }} value={machineCode} readonly />
      {error ? (
        <Text type="danger" style={{ display: "block", marginTop: 8 }}>
          {error}
        </Text>
      ) : null}
    </Modal>
  );
}
