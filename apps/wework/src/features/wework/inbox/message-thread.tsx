"use client";

import { MessageSquare } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@supportflow/shared";
import { Input } from "@supportflow/ui/input";

import type { WeworkConversationSummary, WeworkMessage } from "../types/wework-conversation";

function formatMessageTime(ts: number): string {
  return new Date(ts).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });
}

export interface MessageThreadProps {
  conversation: WeworkConversationSummary | null;
  messages: WeworkMessage[];
}

export function MessageThread({ conversation, messages }: MessageThreadProps) {
  const { t } = useTranslation("console");

  if (!conversation) {
    return (
      <div className="wework-thread-panel flex min-h-0 min-w-0 flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
        <div className="bg-channel-muted/70 flex size-18 items-center justify-center rounded-3xl">
          <MessageSquare className="text-channel/50 size-9" />
        </div>
        <p className="text-foreground text-base font-medium">
          {t("wework_inbox_select_conversation")}
        </p>
      </div>
    );
  }

  return (
    <div className="wework-thread-panel flex min-h-0 min-w-0 flex-1 flex-col">
      <header className="bg-card/88 border-border/70 flex shrink-0 items-center justify-between border-b px-4 py-3 backdrop-blur">
        <div className="min-w-0">
          <h3 className="text-foreground truncate text-sm font-semibold">{conversation.title}</h3>
          <p className="text-muted-foreground truncate text-xs">{conversation.conversationId}</p>
        </div>
        <span className="bg-channel-muted text-channel inline-flex rounded-full px-2 py-0.5 text-[10px] font-medium">
          {conversation.kind === "group" ? t("wework_kind_group") : t("wework_kind_direct")}
        </span>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto px-5 py-5">
        <div className="mx-auto flex max-w-3xl flex-col gap-4">
          {messages.length === 0 ? (
            <p className="text-muted-foreground py-8 text-center text-sm">
              {t("wework_inbox_no_messages")}
            </p>
          ) : (
            messages.map((msg) => {
              const isOutbound = msg.role === "assistant" || msg.role === "operator";
              return (
                <div
                  key={msg.id}
                  className={cn("flex flex-col gap-0.5", isOutbound ? "items-end" : "items-start")}
                >
                  {msg.senderName && !isOutbound ? (
                    <span className="text-muted-foreground px-1 text-[10px]">{msg.senderName}</span>
                  ) : null}
                  <div
                    className={cn(
                      "wework-message-bubble",
                      msg.role === "customer" && "wework-message-bubble--customer",
                      (msg.role === "assistant" || msg.role === "operator") &&
                        "wework-message-bubble--assistant",
                      msg.role === "system" && "wework-message-bubble--system"
                    )}
                  >
                    {msg.content}
                  </div>
                  <span className="text-muted-foreground px-1 text-[10px]">
                    {formatMessageTime(msg.createdAt)}
                  </span>
                </div>
              );
            })
          )}
        </div>
      </div>

      <footer className="bg-card/88 border-border/70 shrink-0 border-t p-3 backdrop-blur">
        <div className="mx-auto max-w-3xl">
          <Input
            type="text"
            disabled
            placeholder={t("wework_inbox_composer_placeholder")}
            className="bg-background border-border text-muted-foreground h-10 w-full rounded-xl text-sm"
          />
          <p className="text-muted-foreground mt-1 text-center text-[10px]">
            {t("wework_inbox_composer_hint")}
          </p>
        </div>
      </footer>
    </div>
  );
}
