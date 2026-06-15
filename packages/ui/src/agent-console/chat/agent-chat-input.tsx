"use client";

import { IconClear, IconCommentStroked } from "@douyinfe/semi-icons";
import { AIChatInput } from "@douyinfe/semi-ui-19";
import type {
  MessageContent,
  Suggestion
} from "@douyinfe/semi-foundation/lib/es/aiChatInput/interface";
import type { ComponentProps } from "react";

import type { ExamplePrompt } from "../constants/example-prompts";

const { Configure } = AIChatInput;

function extractMessageText(payload: MessageContent): string {
  return (payload.inputContents ?? [])
    .map((item) => (item.type === "text" && typeof item.text === "string" ? item.text : ""))
    .join("")
    .trim();
}

function resolveSuggestionText(suggestion: Suggestion): string {
  if (typeof suggestion === "string") {
    return suggestion;
  }
  if (Array.isArray(suggestion)) {
    return suggestion[0] ?? "";
  }
  return suggestion.content ?? "";
}

export interface AgentChatMcpOption {
  value: string;
  label: string;
}

interface AgentChatInputProps {
  generating: boolean;
  canSend?: boolean;
  placeholder?: string;
  suggestions?: string[];
  welcomePrompts?: ExamplePrompt[];
  enabledMcpOptions?: AgentChatMcpOption[];
  onSend: (text: string) => void;
  onStop: () => void;
  onClear: () => void;
  onNewChat?: () => void;
  onOpenMcpConfigure?: () => void;
}

export function AgentChatInput({
  generating,
  canSend,
  placeholder = "输入消息，Enter 发送",
  suggestions,
  welcomePrompts,
  enabledMcpOptions = [],
  onSend,
  onStop,
  onClear,
  onNewChat,
  onOpenMcpConfigure
}: AgentChatInputProps) {
  const handleMessageSend = (payload: MessageContent) => {
    const text = extractMessageText(payload);
    if (text) {
      onSend(text);
    }
  };

  const handleSuggestClick = (suggestion: Suggestion) => {
    const text = resolveSuggestionText(suggestion);
    const item = welcomePrompts?.find((prompt) => prompt.title === text || prompt.text === text);
    onSend(item?.prompt ?? text);
  };

  return (
    <div className="agent-chat-input-wrap">
      <AIChatInput
        round
        generating={generating}
        canSend={canSend}
        showReference={false}
        showUploadFile={false}
        showUploadButton={false}
        showTemplateButton={false}
        placeholder={placeholder}
        sendHotKey="enter"
        suggestions={suggestions as Suggestion[] | undefined}
        onSuggestClick={handleSuggestClick}
        onMessageSend={handleMessageSend}
        onStopGenerate={onStop}
        renderConfigureArea={() => (
          <>
            {onNewChat ? (
              <Configure.Button field="newChat" icon={<IconCommentStroked />} onClick={onNewChat}>
                新对话
              </Configure.Button>
            ) : null}
            <Configure.Button
              field="clear"
              icon={<IconClear />}
              initValue={false}
              onClick={onClear}
            />
            {onOpenMcpConfigure ? (
              <Configure.Button field="mcpConfigure" onClick={onOpenMcpConfigure}>
                MCP
              </Configure.Button>
            ) : null}
            {enabledMcpOptions.length > 0 ? (
              // Configure.Mcp 经 getConfigureItem 包装，运行时需要 field
              <Configure.Mcp
                {...({
                  field: "mcp",
                  options: enabledMcpOptions,
                  showConfigure: Boolean(onOpenMcpConfigure),
                  onConfigureButtonClick: () => {
                    onOpenMcpConfigure?.();
                  }
                } as ComponentProps<typeof Configure.Mcp> & { field: string })}
              />
            ) : null}
          </>
        )}
      />
    </div>
  );
}
