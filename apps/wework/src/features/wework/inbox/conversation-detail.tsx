"use client";

import { useState } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@supportflow/shared";

import type { WeworkConversationSummary } from "../types/wework-conversation";

export interface ConversationDetailProps {
  conversation: WeworkConversationSummary | null;
}

export function ConversationDetail({ conversation }: ConversationDetailProps) {
  const { t } = useTranslation("console");
  const [collapsed, setCollapsed] = useState(false);

  if (collapsed) {
    return (
      <div className="flex w-10 shrink-0 flex-col border-l border-[hsl(var(--border))] bg-white">
        <button
          type="button"
          className="flex flex-1 cursor-pointer items-center justify-center text-slate-400 hover:text-[var(--wework-blue)]"
          onClick={() => setCollapsed(false)}
          aria-label={t("wework_detail_expand")}
        >
          <ChevronLeft className="size-4" />
        </button>
      </div>
    );
  }

  return (
    <aside className="wework-inbox-detail flex min-h-0 shrink-0 flex-col">
      <div className="flex shrink-0 items-center justify-between border-b border-[hsl(var(--border))] px-3 py-2">
        <span className="text-xs font-semibold text-slate-600">{t("wework_detail_title")}</span>
        <button
          type="button"
          className="cursor-pointer rounded p-1 text-slate-400 hover:bg-slate-100"
          onClick={() => setCollapsed(true)}
          aria-label={t("wework_detail_collapse")}
        >
          <ChevronRight className="size-4" />
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto p-3 text-sm">
        {!conversation ? (
          <p className="text-slate-400">{t("wework_detail_empty")}</p>
        ) : (
          <dl className="space-y-3">
            <div>
              <dt className="text-xs text-slate-500">{t("wework_detail_session")}</dt>
              <dd className="mt-0.5 font-mono text-xs break-all text-[#1A2B4A]">
                {conversation.sessionId}
              </dd>
            </div>
            <div>
              <dt className="text-xs text-slate-500">{t("wework_detail_conversation_id")}</dt>
              <dd className="mt-0.5 font-mono text-xs break-all text-[#1A2B4A]">
                {conversation.conversationId}
              </dd>
            </div>
            <div>
              <dt className="text-xs text-slate-500">{t("wework_detail_ai")}</dt>
              <dd className="mt-1">
                <span
                  className={cn(
                    "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                    "bg-[var(--wework-blue-light)] text-[var(--wework-blue)]"
                  )}
                >
                  {t("wework_detail_ai_on")}
                </span>
              </dd>
            </div>
            <p className="text-xs leading-relaxed text-slate-500">{t("wework_detail_mock_hint")}</p>
          </dl>
        )}
      </div>
    </aside>
  );
}
