"use client";

import { Loader2, Search, Users } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@supportflow/shared";

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
      <div className="shrink-0 border-b border-[hsl(var(--border))] p-3">
        <h2 className="text-sm font-semibold text-[#1A2B4A]">{t("wework_inbox_title")}</h2>
        <div className="relative mt-2">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-slate-400" />
          <input
            type="search"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={t("wework_inbox_search")}
            className="w-full rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))] py-1.5 pr-2 pl-8 text-sm outline-none focus:border-[hsl(var(--channel-primary))] focus:ring-1 focus:ring-[hsl(var(--channel-primary))]"
          />
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {loading ? (
          <div className="flex items-center justify-center gap-2 py-12 text-sm text-slate-400">
            <Loader2 className="size-4 animate-spin" />
            <span>{t("wework_inbox_loading")}</span>
          </div>
        ) : conversations.length === 0 ? (
          <p className="px-4 py-8 text-center text-sm text-slate-400">{t("wework_inbox_empty")}</p>
        ) : (
          <ul className="py-1">
            {conversations.map((item) => {
              const isActive = item.conversationId === activeConversationId;
              return (
                <li key={item.conversationId}>
                  <button
                    type="button"
                    className={cn(
                      "wework-conversation-row flex w-full cursor-pointer gap-3 px-3 py-3 text-left",
                      isActive && "active"
                    )}
                    onClick={() => onSelect(item.conversationId)}
                  >
                    <div className="flex size-10 shrink-0 items-center justify-center rounded-full bg-[var(--wework-blue-light)]">
                      {item.kind === "group" ? (
                        <Users className="size-4 text-[var(--wework-blue)]" />
                      ) : (
                        <span className="text-sm font-medium text-[var(--wework-blue)]">
                          {item.title.slice(0, 1)}
                        </span>
                      )}
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className="flex items-baseline justify-between gap-2">
                        <span className="truncate text-sm font-medium text-[#1A2B4A]">
                          {item.title}
                        </span>
                        <span className="shrink-0 text-[10px] text-slate-400">
                          {formatRelativeTime(item.lastActive)}
                        </span>
                      </div>
                      <p className="mt-0.5 truncate text-xs text-slate-500">{item.preview}</p>
                    </div>
                    {(item.unread ?? 0) > 0 ? (
                      <span className="mt-1 flex size-5 shrink-0 items-center justify-center rounded-full bg-[var(--wework-blue)] text-[10px] font-medium text-white">
                        {item.unread! > 9 ? "9+" : item.unread}
                      </span>
                    ) : null}
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </aside>
  );
}
