"use client";

import { useEffect, useState } from "react";
import { Plus, X } from "lucide-react";

import {
  listAgentSessions,
  type AgentSessionSummary
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";

interface SessionPanelProps {
  open: boolean;
  sessionId?: string;
  onClose: () => void;
  onNewChat: () => void;
}

export function SessionPanel({ open, sessionId, onClose, onNewChat }: SessionPanelProps) {
  const [loading, setLoading] = useState(false);
  const [sessions, setSessions] = useState<AgentSessionSummary[]>([]);

  useEffect(() => {
    if (!open) {
      return;
    }
    let mounted = true;
    const load = async () => {
      setLoading(true);
      try {
        const data = await listAgentSessions();
        if (mounted) {
          setSessions(data);
        }
      } catch {
        if (mounted) {
          setSessions([]);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };
    void load();
    return () => {
      mounted = false;
    };
  }, [open, sessionId]);

  if (!open) {
    return null;
  }

  return (
    <>
      <aside className="session-panel">
        <div className="session-panel-header">
          <span className="text-foreground text-sm font-semibold">{"历史会话"}</span>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="text-muted-foreground"
            onClick={onClose}
          >
            <X className="size-3.5" />
          </Button>
        </div>

        <Button type="button" className="session-panel-new" onClick={onNewChat}>
          <Plus className="size-3.5" />
          <span>{"新对话"}</span>
        </Button>

        <div className="session-list">
          {loading ? (
            <p className="session-empty">{"加载会话列表…"}</p>
          ) : sessions.length === 0 ? (
            <p className="session-empty">{"暂无历史会话，发送消息或新建对话后会出现在这里。"}</p>
          ) : (
            sessions.map((session) => (
              <div
                key={session.id}
                className={cn(
                  "rounded-lg px-3 py-2 text-sm",
                  session.id === sessionId
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground"
                )}
              >
                <p className="truncate font-medium">{session.title || "新对话"}</p>
                <p className="truncate font-mono text-xs opacity-70">{session.id}</p>
              </div>
            ))
          )}
        </div>
      </aside>

      <Button
        type="button"
        aria-label="Close session panel"
        variant="ghost"
        className="fixed inset-0 z-30 h-auto w-auto rounded-none bg-black/30 lg:hidden"
        onClick={onClose}
      />
    </>
  );
}
