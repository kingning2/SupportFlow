"use client";

import { useCallback, useEffect, useState } from "react";

import { getAgentConsoleState } from "@/cmd/agent";
import { LocalCacheKey } from "@/enums";
import type { AgentConsoleState } from "@/generated/contracts";

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
        const next = await getAgentConsoleState();
        if (!active) {
          return;
        }
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
