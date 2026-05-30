"use client";

import {
  Conversation,
  ConversationContent,
  ConversationScrollButton
} from "@/components/ai-elements/conversation";
import {
  AssistantMessageBlock,
  UserMessageBlock
} from "@/components/agent-console/chat/message-blocks";
import { WelcomeScreen } from "@/components/agent-console/chat/welcome-screen";
import { isAssistantMessage, type ChatMessage } from "@/types/agent-chat";

interface ChatThreadProps {
  messages: ChatMessage[];
  onSelectPrompt: (text: string) => void;
}

export function ChatThread({ messages, onSelectPrompt }: ChatThreadProps) {
  const isEmpty = messages.length === 0;

  if (isEmpty) {
    return (
      <div className="h-full min-h-0 overflow-y-auto">
        <WelcomeScreen onSelectPrompt={onSelectPrompt} />
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-1 flex-col overflow-hidden">
      <Conversation className="h-full min-h-0 flex-1">
        <ConversationContent className="mx-auto max-w-3xl gap-6 py-6">
          {messages.map((msg) =>
            msg.role === "user" ? (
              <UserMessageBlock key={msg.id} text={msg.text} />
            ) : isAssistantMessage(msg) ? (
              <AssistantMessageBlock key={msg.id} message={msg} />
            ) : null
          )}
        </ConversationContent>
        <ConversationScrollButton />
      </Conversation>
    </div>
  );
}
