"use client";

import { MessageSquare } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@supportflow/shared";

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
      <div className="flex min-h-0 min-w-0 flex-1 flex-col items-center justify-center gap-2 bg-[hsl(var(--muted)/0.35)] p-6 text-center">
        <MessageSquare className="size-10 text-slate-300" />
        <p className="text-sm text-slate-500">{t("wework_inbox_select_conversation")}</p>
      </div>
    );
  }

  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col bg-[hsl(var(--muted)/0.35)]">
      <header className="flex shrink-0 items-center justify-between border-b border-[hsl(var(--border))] bg-white px-4 py-3">
        <div className="min-w-0">
          <h3 className="truncate text-sm font-semibold text-[#1A2B4A]">{conversation.title}</h3>
          <p className="truncate text-xs text-slate-500">{conversation.conversationId}</p>
        </div>
        <span className="rounded-full bg-[var(--wework-blue-light)] px-2 py-0.5 text-[10px] font-medium text-[var(--wework-blue)]">
          {conversation.kind === "group" ? t("wework_kind_group") : t("wework_kind_direct")}
        </span>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto px-4 py-4">
        <div className="mx-auto flex max-w-3xl flex-col gap-3">
          {messages.length === 0 ? (
            <p className="py-8 text-center text-sm text-slate-400">
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
                    <span className="px-1 text-[10px] text-slate-500">{msg.senderName}</span>
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
                  <span className="px-1 text-[10px] text-slate-400">
                    {formatMessageTime(msg.createdAt)}
                  </span>
                </div>
              );
            })
          )}
        </div>
      </div>

      <footer className="shrink-0 border-t border-[hsl(var(--border))] bg-white p-3">
        <div className="mx-auto max-w-3xl">
          <input
            type="text"
            disabled
            placeholder={t("wework_inbox_composer_placeholder")}
            className="w-full cursor-not-allowed rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))] px-3 py-2 text-sm text-slate-400"
          />
          <p className="mt-1 text-center text-[10px] text-slate-400">
            {t("wework_inbox_composer_hint")}
          </p>
        </div>
      </footer>
    </div>
  );
}
