"use client";

import { useState } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";

import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";

import type { WeworkConversationSummary } from "../types/wework-conversation";

export interface ConversationDetailProps {
  conversation: WeworkConversationSummary | null;
}

export function ConversationDetail({ conversation }: ConversationDetailProps) {
  const [collapsed, setCollapsed] = useState(false);

  if (collapsed) {
    return (
      <div className="detail bg-card/88 flex w-10 shrink-0 flex-col border-l border-[hsl(var(--border))]">
        <Button
          type="button"
          variant="ghost"
          className="text-muted-foreground hover:text-channel flex flex-1 items-center justify-center rounded-none"
          onClick={() => setCollapsed(false)}
          aria-label={"展开详情"}
        >
          <ChevronLeft className="size-4" />
        </Button>
      </div>
    );
  }

  return (
    <aside className="inbox-detail flex min-h-0 shrink-0 flex-col">
      <div className="border-border/70 flex shrink-0 items-center justify-between border-b px-3 py-3">
        <span className="text-foreground text-sm font-semibold">{"会话详情"}</span>
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          className="text-muted-foreground"
          onClick={() => setCollapsed(true)}
          aria-label={"收起详情"}
        >
          <ChevronRight className="size-4" />
        </Button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-3 py-4 text-sm">
        {!conversation ? (
          <p className="text-muted-foreground">{"选择会话后显示详情"}</p>
        ) : (
          <dl className="space-y-3">
            <div>
              <dt className="text-muted-foreground text-xs">{"Agent Session"}</dt>
              <dd className="text-foreground mt-0.5 font-mono text-xs break-all">
                {conversation.sessionId}
              </dd>
            </div>
            <div>
              <dt className="text-muted-foreground text-xs">{"Conversation ID"}</dt>
              <dd className="text-foreground mt-0.5 font-mono text-xs break-all">
                {conversation.conversationId}
              </dd>
            </div>
            <div>
              <dt className="text-muted-foreground text-xs">{"群 AI"}</dt>
              <dd className="mt-1">
                <span
                  className={cn(
                    "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                    "bg-channel-muted text-channel"
                  )}
                >
                  {"已启用"}
                </span>
              </dd>
            </div>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {"当前为演示数据；一群一会话，session 映射为 wework:{conversationId}。"}
            </p>
          </dl>
        )}
      </div>
    </aside>
  );
}
