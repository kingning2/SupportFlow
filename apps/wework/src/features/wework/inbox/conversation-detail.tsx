"use client";

import { useState } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";

import type { WeworkConversationSummary } from "../types/wework-conversation";

export interface ConversationDetailProps {
  conversation: WeworkConversationSummary | null;
}

export function ConversationDetail({ conversation }: ConversationDetailProps) {
  const { t } = useTranslation("console");
  const [collapsed, setCollapsed] = useState(false);

  if (collapsed) {
    return (
      <div className="wework-detail-panel bg-card/88 flex w-10 shrink-0 flex-col border-l border-[hsl(var(--border))]">
        <Button
          type="button"
          variant="ghost"
          className="text-muted-foreground hover:text-channel flex flex-1 items-center justify-center rounded-none"
          onClick={() => setCollapsed(false)}
          aria-label={t("wework_detail_expand")}
        >
          <ChevronLeft className="size-4" />
        </Button>
      </div>
    );
  }

  return (
    <aside className="wework-inbox-detail flex min-h-0 shrink-0 flex-col">
      <div className="border-border/70 flex shrink-0 items-center justify-between border-b px-3 py-3">
        <span className="text-foreground text-sm font-semibold">{t("wework_detail_title")}</span>
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          className="text-muted-foreground"
          onClick={() => setCollapsed(true)}
          aria-label={t("wework_detail_collapse")}
        >
          <ChevronRight className="size-4" />
        </Button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-3 py-4 text-sm">
        {!conversation ? (
          <p className="text-muted-foreground">{t("wework_detail_empty")}</p>
        ) : (
          <dl className="space-y-3">
            <div>
              <dt className="text-muted-foreground text-xs">{t("wework_detail_session")}</dt>
              <dd className="text-foreground mt-0.5 font-mono text-xs break-all">
                {conversation.sessionId}
              </dd>
            </div>
            <div>
              <dt className="text-muted-foreground text-xs">
                {t("wework_detail_conversation_id")}
              </dt>
              <dd className="text-foreground mt-0.5 font-mono text-xs break-all">
                {conversation.conversationId}
              </dd>
            </div>
            <div>
              <dt className="text-muted-foreground text-xs">{t("wework_detail_ai")}</dt>
              <dd className="mt-1">
                <span
                  className={cn(
                    "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                    "bg-channel-muted text-channel"
                  )}
                >
                  {t("wework_detail_ai_on")}
                </span>
              </dd>
            </div>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {t("wework_detail_mock_hint")}
            </p>
          </dl>
        )}
      </div>
    </aside>
  );
}
