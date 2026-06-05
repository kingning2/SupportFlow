"use client";

import { useCallback, useEffect, useMemo, useState } from "react";

import { LocalCacheKey } from "@supportflow/shared/tauri-bridge/enums";

import { MOCK_WEWORK_CONVERSATIONS, MOCK_WEWORK_MESSAGES } from "../constants/mock-data";
import type {
  WeworkConnectionStatus,
  WeworkConversationSummary,
  WeworkMessage
} from "../types/wework-conversation";

function readStoredConversationId(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  return localStorage.getItem(LocalCacheKey.WeworkActiveConversationId);
}

function persistConversationId(id: string | null) {
  if (typeof window === "undefined") {
    return;
  }
  if (id) {
    localStorage.setItem(LocalCacheKey.WeworkActiveConversationId, id);
  } else {
    localStorage.removeItem(LocalCacheKey.WeworkActiveConversationId);
  }
}

export interface UseWeworkInboxOptions {
  /** 通道已连接时为 ready；未接 IPC 前由壳层传入 */
  connectionStatus: WeworkConnectionStatus;
}

export function useWeworkInbox({ connectionStatus }: UseWeworkInboxOptions) {
  const [conversations, setConversations] = useState<WeworkConversationSummary[]>([]);
  const [messagesByConversation, setMessagesByConversation] = useState<
    Record<string, WeworkMessage[]>
  >({});
  const [activeConversationId, setActiveConversationIdState] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(true);

  const loadMock = useCallback(async () => {
    setLoading(true);
    await new Promise((r) => setTimeout(r, 120));
    const sorted = [...MOCK_WEWORK_CONVERSATIONS].sort((a, b) => b.lastActive - a.lastActive);
    setConversations(sorted);
    setMessagesByConversation({ ...MOCK_WEWORK_MESSAGES });

    const stored = readStoredConversationId();
    const fallback = sorted[0]?.conversationId ?? null;
    const nextId = stored && sorted.some((c) => c.conversationId === stored) ? stored : fallback;
    setActiveConversationIdState(nextId);
    if (nextId) {
      persistConversationId(nextId);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    if (connectionStatus !== "ready") {
      return;
    }
    let cancelled = false;
    void (async () => {
      setLoading(true);
      await new Promise((r) => setTimeout(r, 120));
      if (cancelled) {
        return;
      }
      const sorted = [...MOCK_WEWORK_CONVERSATIONS].sort((a, b) => b.lastActive - a.lastActive);
      setConversations(sorted);
      setMessagesByConversation({ ...MOCK_WEWORK_MESSAGES });

      const stored = readStoredConversationId();
      const fallback = sorted[0]?.conversationId ?? null;
      const nextId = stored && sorted.some((c) => c.conversationId === stored) ? stored : fallback;
      setActiveConversationIdState(nextId);
      if (nextId) {
        persistConversationId(nextId);
      }
      setLoading(false);
    })();
    return () => {
      cancelled = true;
    };
  }, [connectionStatus]);

  const setActiveConversationId = useCallback((id: string | null) => {
    setActiveConversationIdState(id);
    persistConversationId(id);
  }, []);

  const filteredConversations = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) {
      return conversations;
    }
    return conversations.filter(
      (c) => c.title.toLowerCase().includes(q) || c.preview.toLowerCase().includes(q)
    );
  }, [conversations, searchQuery]);

  const activeConversation = useMemo(
    () => conversations.find((c) => c.conversationId === activeConversationId) ?? null,
    [conversations, activeConversationId]
  );

  const activeMessages = useMemo(
    () => (activeConversationId ? (messagesByConversation[activeConversationId] ?? []) : []),
    [activeConversationId, messagesByConversation]
  );

  return {
    loading,
    conversations: filteredConversations,
    activeConversationId,
    activeConversation,
    activeMessages,
    searchQuery,
    setSearchQuery,
    setActiveConversationId,
    refresh: loadMock
  };
}
