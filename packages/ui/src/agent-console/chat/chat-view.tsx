"use client";

import { ChatComposer } from "../chat/chat-composer";
import { ChatThread } from "../chat/chat-thread";
import { useAgentChat } from "../hooks/use-agent-chat";
import type { AgentConsoleState } from "@supportflow/shared/contracts";
import { useTranslation } from "react-i18next";

interface ChatViewProps {
  sessionId?: string;
  consoleState: AgentConsoleState | null;
  onNewSession: () => void;
}

export function ChatView({ sessionId, consoleState, onNewSession }: ChatViewProps) {
  const { t } = useTranslation("console");
  const { messages, isStreaming, sendMessage, cancel, clearContext, resetMessages } =
    useAgentChat(sessionId);

  const activeProvider = consoleState?.providers.find((p) => p.isActive);
  const apiKeyMissing = activeProvider !== undefined && !activeProvider.configured;

  const handleNewChat = () => {
    resetMessages();
    onNewSession();
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      {apiKeyMissing ? (
        <div className="border-b border-amber-500/30 bg-amber-500/10 px-4 py-2 text-sm text-amber-800 dark:text-amber-200">
          {t("api_key_not_configured")}
          {consoleState?.workspaceDir ? (
            <span className="mt-1 block font-mono text-xs opacity-80">
              {consoleState.workspaceDir}
              {consoleState.configPath ? ` / ${consoleState.configPath}` : ""}
            </span>
          ) : null}
        </div>
      ) : null}
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ChatThread
          messages={messages}
          onSelectPrompt={(text) => {
            void sendMessage(text);
          }}
        />
      </div>
      <ChatComposer
        isStreaming={isStreaming}
        onSend={sendMessage}
        onCancel={cancel}
        onClearContext={() => {
          void clearContext();
        }}
        onNewChat={handleNewChat}
      />
    </div>
  );
}
