"use client";

import { AIChatDialogue, Banner, Button, Space, Typography } from "@douyinfe/semi-ui-19";
import { useCallback, useMemo, useState, type ReactNode } from "react";

import { EXAMPLE_PROMPTS } from "../constants/example-prompts";
import type { ExamplePrompt } from "../constants/example-prompts";
import { useAgentChat } from "../hooks/use-agent-chat";
import type { AgentConsoleState } from "@supportflow/shared/contracts";

import { AgentChatInput } from "./agent-chat-input";
import { ChatMcpSidebar } from "./chat-mcp-sidebar";
import { getChatEnabledMcps } from "./chat-mcp-storage";
import { mapChatMessagesToDialogue } from "./map-dialogue-messages";

const { Text } = Typography;

const DIALOGUE_ROLE_CONFIG = {
  user: {
    name: "我"
  },
  assistant: {
    name: "AI 助手"
  }
};

const WELCOME_MESSAGE_CONTENT = "你好，我是 AI 助手。你可以直接提问，或点击下方快捷提示开始。";

interface ChatTopBanner {
  type?: "info" | "warning" | "danger";
  description: ReactNode;
}

interface ChatProps {
  sessionId?: string;
  consoleState: AgentConsoleState | null;
  onNewSession: () => void;
  topBanner?: ChatTopBanner;
  welcomePrompts?: ExamplePrompt[];
  showMcpSidebar?: boolean;
}

export function Chat({
  sessionId,
  consoleState,
  onNewSession,
  topBanner,
  welcomePrompts = EXAMPLE_PROMPTS,
  showMcpSidebar = true
}: ChatProps) {
  const { messages, isStreaming, sendMessage, cancel, clearContext, resetMessages } =
    useAgentChat(sessionId);
  const [mcpSidebarExpanded, setMcpSidebarExpanded] = useState(false);

  const activeProvider = consoleState?.providers.find((p) => p.isActive);
  const isOllama = consoleState?.botType?.toLowerCase() === "ollama";
  const apiKeyMissing = !isOllama && activeProvider !== undefined && !activeProvider.configured;

  const dialogueChats = useMemo(() => {
    const mapped = mapChatMessagesToDialogue(messages);
    if (mapped.length > 0) {
      return mapped;
    }
    return [
      {
        id: "welcome",
        role: "assistant" as const,
        name: "AI 助手",
        content: WELCOME_MESSAGE_CONTENT,
        createdAt: Date.now(),
        status: "completed" as const
      }
    ];
  }, [messages]);

  const hints = useMemo(
    () => (messages.length === 0 ? welcomePrompts.map((item) => item.title) : []),
    [messages.length, welcomePrompts]
  );

  const enabledMcpOptions = useMemo(() => {
    const mcpStatus = consoleState?.mcpStatus ?? {};
    const enabled = getChatEnabledMcps(sessionId, mcpStatus);
    return enabled.map((name) => ({ value: name, label: name }));
  }, [consoleState?.mcpStatus, sessionId]);

  const handleNewChat = useCallback(() => {
    resetMessages();
    onNewSession();
  }, [onNewSession, resetMessages]);

  const handleMessageSend = useCallback(
    (content: string) => {
      void sendMessage(content);
    },
    [sendMessage]
  );

  const handleClear = useCallback(() => {
    void clearContext();
  }, [clearContext]);

  const handleHintClick = useCallback(
    (hint: string) => {
      const item = welcomePrompts.find((prompt) => prompt.title === hint);
      void sendMessage(item?.prompt ?? item?.text ?? hint);
    },
    [sendMessage, welcomePrompts]
  );

  const renderHintBox = useCallback(
    (props: { content: string; index: number; onHintClick: () => void }) => {
      const prompt = welcomePrompts.find((item) => item.title === props.content);
      return (
        <Button
          key={props.content}
          type="tertiary"
          theme="light"
          className="agent-chat-hint-item"
          onClick={props.onHintClick}
        >
          {prompt ? (
            <span
              className="agent-chat-hint-item__icon"
              style={{ background: prompt.iconBg, color: prompt.iconColor }}
              aria-hidden
            />
          ) : null}
          <span className="agent-chat-hint-item__text">{props.content}</span>
        </Button>
      );
    },
    [welcomePrompts]
  );

  return (
    <div className="agent-chat-shell">
      <div className="agent-chat-main">
        {apiKeyMissing ? (
          <Banner
            closeIcon={null}
            description={
              consoleState?.workspaceDir ? (
                <Space vertical align="start" spacing={4}>
                  <Text>
                    {consoleState.workspaceDir}
                    {consoleState.configPath ? ` / ${consoleState.configPath}` : ""}
                  </Text>
                </Space>
              ) : null
            }
            fullMode={false}
            style={{ flexShrink: 0 }}
            title="当前模型厂商未配置 API Key。请前往「AI 配置」添加凭据。"
            type="warning"
          />
        ) : null}

        {topBanner ? (
          <Banner
            fullMode={false}
            bordered
            closeIcon={null}
            type={topBanner.type ?? "info"}
            description={topBanner.description}
            style={{ flexShrink: 0, margin: "12px 16px 0" }}
          />
        ) : null}

        <div className="agent-chat-body">
          <AIChatDialogue
            className="agent-chat-dialogue"
            mode="bubble"
            align="leftRight"
            chats={dialogueChats}
            hints={hints}
            hintCls="agent-chat-hints"
            renderHintBox={renderHintBox}
            onHintClick={handleHintClick}
            roleConfig={DIALOGUE_ROLE_CONFIG}
            showReference={false}
            dialogueRenderConfig={{
              renderDialogueAction: () => null
            }}
          />

          <AgentChatInput
            generating={isStreaming}
            canSend={!apiKeyMissing}
            welcomePrompts={welcomePrompts}
            enabledMcpOptions={enabledMcpOptions}
            onSend={handleMessageSend}
            onStop={cancel}
            onClear={handleClear}
            onNewChat={handleNewChat}
            onOpenMcpConfigure={() => {
              setMcpSidebarExpanded(true);
            }}
          />
        </div>
      </div>

      {showMcpSidebar ? (
        <ChatMcpSidebar
          sessionId={sessionId}
          consoleState={consoleState}
          expanded={mcpSidebarExpanded}
          onExpand={() => {
            setMcpSidebarExpanded(true);
          }}
          onCollapse={() => {
            setMcpSidebarExpanded(false);
          }}
        />
      ) : null}
    </div>
  );
}
