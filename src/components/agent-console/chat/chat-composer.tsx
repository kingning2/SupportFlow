"use client";

import type { FormEvent } from "react";
import { Eraser, MessageSquarePlus } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  PromptInput,
  PromptInputBody,
  PromptInputFooter,
  PromptInputSubmit,
  PromptInputTextarea,
  PromptInputTools,
  PromptInputButton
} from "@/components/ai-elements/prompt-input";

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
  const { t } = useTranslation("console");

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
          <PromptInputTextarea placeholder={t("input_placeholder")} />
        </PromptInputBody>
        <PromptInputFooter>
          <PromptInputTools>
            <PromptInputButton tooltip={t("tip_new_chat")} onClick={onNewChat}>
              <MessageSquarePlus className="size-4" />
            </PromptInputButton>
            <PromptInputButton tooltip={t("tip_clear_context")} onClick={onClearContext}>
              <Eraser className="size-4" />
            </PromptInputButton>
          </PromptInputTools>
          <PromptInputSubmit
            status={isStreaming ? "streaming" : undefined}
            onStop={onCancel}
            className="bg-[#35A85B] text-white hover:bg-[#228547]"
          />
        </PromptInputFooter>
      </PromptInput>
    </div>
  );
}
