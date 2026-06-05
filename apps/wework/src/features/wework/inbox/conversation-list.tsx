"use client";

import { Loader2, Search, Users } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";
import { Input } from "@supportflow/ui/input";

import type { WeworkConversationSummary } from "../types/wework-conversation";

function formatRelativeTime(ts: number): string {
  const diff = Date.now() - ts;
  if (diff < 60_000) {
    return "刚刚";
  }
  if (diff < 3_600_000) {
    return `${Math.floor(diff / 60_000)} 分钟前`;
  }
  if (diff < 86_400_000) {
    return `${Math.floor(diff / 3_600_000)} 小时前`;
  }
  return `${Math.floor(diff / 86_400_000)} 天前`;
}

export interface ConversationListProps {
  loading: boolean;
  conversations: WeworkConversationSummary[];
  activeConversationId: string | null;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onSelect: (conversationId: string) => void;
}

export function ConversationList({
  loading,
  conversations,
  activeConversationId,
  searchQuery,
  onSearchChange,
  onSelect
}: ConversationListProps) {
  const { t } = useTranslation("console");

  return (
    <aside className="wework-inbox-list flex min-h-0 shrink-0 flex-col">
      <div className="border-border/70 shrink-0 border-b px-3 py-3">
        <h2 className="text-foreground text-base font-semibold tracking-tight">
          {t("wework_inbox_title")}
        </h2>
        <div className="relative mt-2">
          <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2" />
          <Input
            type="search"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={t("wework_inbox_search")}
            className="bg-background border-border h-9 w-full rounded-xl pr-2 pl-8 text-sm shadow-none"
          />
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {loading ? (
          <div className="text-muted-foreground flex items-center justify-center gap-2 py-12 text-sm">
            <Loader2 className="size-4 animate-spin" />
            <span>{t("wework_inbox_loading")}</span>
          </div>
        ) : conversations.length === 0 ? (
          <p className="text-muted-foreground px-4 py-8 text-center text-sm">
            {t("wework_inbox_empty")}
          </p>
        ) : (
          <ul className="space-y-1 px-2 py-2">
            {conversations.map((item) => {
              const isActive = item.conversationId === activeConversationId;
              return (
                <li key={item.conversationId}>
                  <Button
                    type="button"
                    variant="ghost"
                    className={cn(
                      "wework-conversation-row flex h-auto w-full items-start gap-3 rounded-2xl px-3 py-3 text-left",
                      isActive && "active"
                    )}
                    onClick={() => onSelect(item.conversationId)}
                  >
                    <div className="bg-channel-muted flex size-10 shrink-0 items-center justify-center rounded-full">
                      {item.kind === "group" ? (
                        <Users className="text-channel size-4" />
                      ) : (
                        <span className="text-channel text-sm font-medium">
                          {item.title.slice(0, 1)}
                        </span>
                      )}
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className="flex items-baseline justify-between gap-2">
                        <span className="text-foreground truncate text-sm leading-5 font-medium">
                          {item.title}
                        </span>
                        <span className="text-muted-foreground shrink-0 text-[10px]">
                          {formatRelativeTime(item.lastActive)}
                        </span>
                      </div>
                      <p className="text-muted-foreground mt-1 truncate text-xs leading-5">
                        {item.preview}
                      </p>
                    </div>
                    {(item.unread ?? 0) > 0 ? (
                      <span className="bg-channel text-channel-foreground mt-1 flex size-5 shrink-0 items-center justify-center rounded-full text-[10px] font-medium">
                        {item.unread! > 9 ? "9+" : item.unread}
                      </span>
                    ) : null}
                  </Button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </aside>
  );
}
