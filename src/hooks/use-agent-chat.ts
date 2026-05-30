"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { cancelAgentMessage, clearAgentContext, sendAgentMessage } from "@/cmd/agent";
import { TauriEvent } from "@/enums";
import type { AgentRunFinished, AgentStreamChunk } from "@/generated/contracts";
import {
  applyStreamChunk,
  createAssistantMessage,
  finalizeAssistant
} from "@/lib/agent-console/stream-reducer";
import type { ChatMessage } from "@/types/agent-chat";
import { tauriOn } from "@/utils/tauri-event";

function createUserMessage(text: string): ChatMessage {
  return {
    id: crypto.randomUUID(),
    role: "user",
    text
  };
}

export function useAgentChat(sessionId: string | undefined) {
  const { t } = useTranslation("console");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const activeRequestIdRef = useRef<string | null>(null);

  useEffect(() => {
    const unlistenChunk = tauriOn<AgentStreamChunk>(TauriEvent.AgentStreamChunk, (event) => {
      const chunk = event.payload;
      if (!chunk?.requestId) {
        return;
      }
      setMessages((prev) => applyStreamChunk(prev, chunk));
    });

    const unlistenFinished = tauriOn<AgentRunFinished>(TauriEvent.AgentRunFinished, (event) => {
      const payload = event.payload;
      if (!payload?.requestId) {
        return;
      }
      if (activeRequestIdRef.current === payload.requestId) {
        activeRequestIdRef.current = null;
        setIsStreaming(false);
      }
      setMessages((prev) => finalizeAssistant(prev, payload.requestId, payload.error ?? undefined));
    });

    return () => {
      unlistenChunk();
      unlistenFinished();
    };
  }, []);

  const sendMessage = useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (!trimmed || isStreaming) {
        return;
      }

      setMessages((prev) => [...prev, createUserMessage(trimmed)]);

      try {
        const response = await sendAgentMessage({
          message: trimmed,
          sessionId
        });
        activeRequestIdRef.current = response.requestId;
        setIsStreaming(true);
        setMessages((prev) => [...prev, createAssistantMessage(response.requestId)]);
      } catch {
        setIsStreaming(false);
        setMessages((prev) => [
          ...prev,
          {
            id: crypto.randomUUID(),
            role: "assistant",
            requestId: "",
            reasoning: "",
            reasoningStreaming: false,
            toolSteps: [],
            content: t("send_failed"),
            streaming: false,
            cancelled: false
          }
        ]);
      }
    },
    [isStreaming, sessionId, t]
  );

  const cancel = useCallback(() => {
    const requestId = activeRequestIdRef.current;
    if (!requestId) {
      return;
    }
    void cancelAgentMessage(requestId);
  }, []);

  const clearContext = useCallback(async () => {
    await clearAgentContext();
    setMessages([]);
  }, []);

  const resetMessages = useCallback(() => {
    setMessages([]);
    activeRequestIdRef.current = null;
    setIsStreaming(false);
  }, []);

  return {
    messages,
    isStreaming,
    sendMessage,
    cancel,
    clearContext,
    resetMessages
  };
}
