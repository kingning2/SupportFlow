"use client";

import { useCallback, useEffect, useState } from "react";

import { resolveActiveSavedAccount } from "../accounts/storage";
import type { WeworkSavedAccount } from "../accounts/types";
import type { WeworkConnectionStatus } from "../types/wework-conversation";

export function useActiveWeworkAccount(connectionStatus: WeworkConnectionStatus) {
  const [resolvedAccount, setResolvedAccount] = useState<WeworkSavedAccount | null>(null);
  const account = connectionStatus === "ready" ? resolvedAccount : null;

  const refresh = useCallback(() => {
    if (connectionStatus !== "ready") {
      return;
    }
    void resolveActiveSavedAccount().then(setResolvedAccount);
  }, [connectionStatus]);

  useEffect(() => {
    if (connectionStatus !== "ready") {
      return;
    }
    let cancelled = false;
    void resolveActiveSavedAccount().then((value) => {
      if (!cancelled) {
        setResolvedAccount(value);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [connectionStatus]);

  return { account, refreshActiveAccount: refresh };
}
