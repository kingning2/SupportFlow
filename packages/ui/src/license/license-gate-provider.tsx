"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode
} from "react";

import {
  getLicenseStatus,
  type LicenseStatusDto
} from "@supportflow/shared/tauri-bridge/cmd/license";
import {
  isLicenseLockedError,
  parseLicenseLockReason
} from "@supportflow/shared/tauri-bridge/license-error";
import { subscribeInvokeErrors } from "@supportflow/shared/tauri-bridge/cmd";

export type LicenseGateContextValue = {
  loading: boolean;
  valid: boolean;
  status: LicenseStatusDto | null;
  reasonLabel: string;
  refresh: () => Promise<void>;
  applyStatus: (next: LicenseStatusDto) => void;
};

const LicenseGateContext = createContext<LicenseGateContextValue | null>(null);

export function useLicenseGate(): LicenseGateContextValue {
  const ctx = useContext(LicenseGateContext);
  if (!ctx) {
    throw new Error("useLicenseGate must be used within <LicenseGateProvider />");
  }
  return ctx;
}

export function useOptionalLicenseGate(): LicenseGateContextValue | null {
  return useContext(LicenseGateContext);
}

type LicenseGateProviderProps = {
  children: ReactNode;
};

export function LicenseGateProvider({ children }: LicenseGateProviderProps) {
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState<LicenseStatusDto | null>(null);

  const refresh = useCallback(async () => {
    try {
      const next = await getLicenseStatus();
      setStatus(next);
    } catch {
      setStatus({ machineCode: "", valid: false, reason: "无法读取订阅状态" });
    } finally {
      setLoading(false);
    }
  }, []);

  const applyStatus = useCallback((next: LicenseStatusDto) => {
    setStatus(next);
    setLoading(false);
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    return subscribeInvokeErrors((error) => {
      if (!isLicenseLockedError(error)) {
        return;
      }
      const reason = parseLicenseLockReason(error);
      setStatus((prev) => ({
        machineCode: prev?.machineCode ?? "",
        valid: false,
        reason: reason ?? prev?.reason ?? "license locked"
      }));
      setLoading(false);
    });
  }, []);

  const value = useMemo<LicenseGateContextValue>(
    () => ({
      loading,
      valid: status?.valid === true,
      status,
      reasonLabel: status?.reason ?? "",
      refresh,
      applyStatus
    }),
    [applyStatus, loading, refresh, status]
  );

  return <LicenseGateContext.Provider value={value}>{children}</LicenseGateContext.Provider>;
}
