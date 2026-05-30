"use client";

import { Plus, X } from "lucide-react";
import { useTranslation } from "react-i18next";

interface SessionPanelProps {
  open: boolean;
  sessionId?: string;
  onClose: () => void;
  onNewChat: () => void;
}

export function SessionPanel({ open, sessionId, onClose, onNewChat }: SessionPanelProps) {
  const { t } = useTranslation("console");

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
          {sessionId ? (
            <div className="rounded-lg bg-slate-200/80 px-3 py-2 text-sm text-slate-700 dark:bg-white/10 dark:text-slate-200">
              {t("untitled_session")}
            </div>
          ) : (
            <p className="session-empty">{t("session_list_hint")}</p>
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
