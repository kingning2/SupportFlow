"use client";

import { useEffect, useState } from "react";
import { Plus, X } from "lucide-react";
import { useTranslation } from "react-i18next";

import { listAgentSessions, type AgentSessionSummary } from "@/cmd/agent";
import { cn } from "@/lib/utils";

interface SessionPanelProps {
  open: boolean;
  sessionId?: string;
  onClose: () => void;
  onNewChat: () => void;
}

export function SessionPanel({ open, sessionId, onClose, onNewChat }: SessionPanelProps) {
  const { t } = useTranslation("console");
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
          <span className="text-sm font-semibold text-slate-700 dark:text-slate-300">
            {t("session_history")}
          </span>
          <button
            type="button"
            className="flex size-7 cursor-pointer items-center justify-center rounded-md text-slate-400 hover:bg-slate-100 hover:text-slate-600 dark:hover:bg-white/8 dark:hover:text-slate-200"
            onClick={onClose}
          >
            <X className="size-3.5" />
          </button>
        </div>

        <button type="button" className="session-panel-new" onClick={onNewChat}>
          <Plus className="size-3.5" />
          <span>{t("new_chat")}</span>
        </button>

        <div className="session-list">
          {loading ? (
            <p className="session-empty">{t("session_loading")}</p>
          ) : sessions.length === 0 ? (
            <p className="session-empty">{t("session_list_hint")}</p>
          ) : (
            sessions.map((session) => (
              <div
                key={session.id}
                className={cn(
                  "rounded-lg px-3 py-2 text-sm",
                  session.id === sessionId
                    ? "bg-slate-200/80 text-slate-700 dark:bg-white/10 dark:text-slate-200"
                    : "text-slate-600 dark:text-slate-400"
                )}
              >
                <p className="truncate font-medium">{session.title || t("untitled_session")}</p>
                <p className="truncate font-mono text-xs opacity-70">{session.id}</p>
              </div>
            ))
          )}
        </div>
      </aside>

      <button
        type="button"
        aria-label="Close session panel"
        className="fixed inset-0 z-30 bg-black/30 lg:hidden"
        onClick={onClose}
      />
    </>
  );
}
