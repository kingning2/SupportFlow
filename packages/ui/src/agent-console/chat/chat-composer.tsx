"use client";

import type { FormEvent } from "react";
import { Eraser, MessageSquarePlus } from "lucide-react";

import {
  PromptInput,
  PromptInputBody,
  PromptInputFooter,
  PromptInputSubmit,
  PromptInputTextarea,
  PromptInputTools,
  PromptInputButton
} from "../ai-elements/prompt-input";

interface ChatComposerProps {
  isStreaming: boolean;
  onSend: (text: string) => void | Promise<void>;
  onCancel: () => void;
  onClearContext: () => void;
  onNewChat: () => void;
}

export function ChatComposer({
  isStreaming,
  onSend,
  onCancel,
  onClearContext,
  onNewChat
}: ChatComposerProps) {
  const handleSubmit = async (message: { text: string }, event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const text = message.text.trim();
    if (!text) {
      return;
    }
    await onSend(text);
  };

  return (
    <div className="composer-shell shrink-0 px-4 py-3">
      <PromptInput className="mx-auto max-w-3xl" onSubmit={handleSubmit}>
        <PromptInputBody>
          <PromptInputTextarea placeholder={"输入消息，或输入 / 使用指令"} />
        </PromptInputBody>
        <PromptInputFooter>
          <PromptInputTools>
            <PromptInputButton tooltip={"新建对话"} onClick={onNewChat}>
              <MessageSquarePlus className="size-4" />
            </PromptInputButton>
            <PromptInputButton tooltip={"清除上下文"} onClick={onClearContext}>
              <Eraser className="size-4" />
            </PromptInputButton>
          </PromptInputTools>
          <PromptInputSubmit
            status={isStreaming ? "streaming" : undefined}
            onStop={onCancel}
            className="bg-primary text-primary-foreground hover:bg-primary/90"
          />
        </PromptInputFooter>
      </PromptInput>
    </div>
  );
}
