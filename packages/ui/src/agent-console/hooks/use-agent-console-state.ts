"use client";

import { useCallback, useEffect, useState } from "react";

import { getAgentConsoleState } from "@supportflow/shared/tauri-bridge/cmd/agent";
import { LocalCacheKey } from "@supportflow/shared/tauri-bridge/enums";
import type { AgentConsoleState } from "@supportflow/shared/contracts";

export function useAgentConsoleState() {
  const [state, setState] = useState<AgentConsoleState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await getAgentConsoleState();
      setState(next);
      localStorage.setItem(LocalCacheKey.AgentSessionId, next.sessionId);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;

    void (async () => {
      try {
        if (!active) {
          return;
        }
        const next = await getAgentConsoleState();
        setState(next);
        localStorage.setItem(LocalCacheKey.AgentSessionId, next.sessionId);
      } catch (err) {
        if (!active) {
          return;
        }
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    })();

    return () => {
      active = false;
    };
  }, []);

  return { state, setState, loading, error, reload };
}
