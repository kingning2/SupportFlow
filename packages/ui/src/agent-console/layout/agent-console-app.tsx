"use client";

import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { ChatView } from "../chat/chat-view";
import {
  getSidebarNavGroups,
  PLACEHOLDER_CONSOLE_VIEWS,
  SidebarGroupId
} from "../constants/sidebar-nav";
import { ConsoleHeader } from "../layout/console-header";
import { ConsoleSidebar } from "../layout/console-sidebar";
import { SessionPanel } from "../layout/session-panel";
import { PlaceholderView } from "../shared/console-brand";
import { ChannelsView } from "../views/channels-view";
import { ConfigView } from "../views/config-view";
import { KnowledgeView } from "../views/knowledge-view";
import { LogsView } from "../views/logs-view";
import { MemoryView } from "../views/memory-view";
import { ModelsView } from "../views/models-view";
import { SkillsView } from "../views/skills-view";
import { TasksView } from "../views/tasks-view";
import { TooltipProvider } from "@supportflow/ui/tooltip";
import { newAgentSession } from "@supportflow/shared/tauri-bridge/cmd/agent";
import { ConsoleView, LocalCacheKey } from "@supportflow/shared/tauri-bridge/enums";
import { useAgentConsoleState } from "../hooks/use-agent-console-state";
import { getDevChannel } from "../lib/agent-console/dev-channel";
import {
  applyConsoleTheme,
  readConsoleTheme,
  toggleConsoleTheme,
  type ConsoleTheme
} from "../lib/agent-console/theme-sync";

const DEFAULT_OPEN_GROUPS: Record<SidebarGroupId, boolean> = {
  [SidebarGroupId.Chat]: true,
  [SidebarGroupId.Manage]: true,
  [SidebarGroupId.Monitor]: true
};

export function AgentConsoleApp() {
  const { t } = useTranslation("console");
  const { state, setState, loading, error } = useAgentConsoleState();
  const devChannel = useMemo(() => getDevChannel(), []);
  const sidebarNavGroups = useMemo(() => getSidebarNavGroups(devChannel), [devChannel]);

  const [activeView, setActiveView] = useState<ConsoleView>(ConsoleView.Chat);
  const [theme, setTheme] = useState<ConsoleTheme>(() => {
    const initial = readConsoleTheme();
    applyConsoleTheme(initial);
    return initial;
  });
  const [sessionPanelOpen, setSessionPanelOpen] = useState(false);
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);
  const [openGroups, setOpenGroups] =
    useState<Record<SidebarGroupId, boolean>>(DEFAULT_OPEN_GROUPS);

  const handleToggleTheme = useCallback(() => {
    setTheme(toggleConsoleTheme());
  }, []);

  const handleNewSession = useCallback(async () => {
    try {
      const sessionId = await newAgentSession();
      localStorage.setItem(LocalCacheKey.AgentSessionId, sessionId);
      if (state) {
        setState({ ...state, sessionId });
      }
    } catch {
      // keep current session
    }
  }, [state, setState]);

  const toggleGroup = useCallback((groupId: SidebarGroupId) => {
    setOpenGroups((prev) => ({ ...prev, [groupId]: !prev[groupId] }));
  }, []);

  const viewContent = useMemo(() => {
    if (activeView === ConsoleView.Chat) {
      return (
        <ChatView
          sessionId={state?.sessionId}
          consoleState={state}
          onNewSession={() => void handleNewSession()}
        />
      );
    }
    if (activeView === ConsoleView.Config) {
      return <ConfigView state={state} />;
    }
    if (activeView === ConsoleView.Models) {
      return (
        <ModelsView
          state={state}
          onRefresh={(next) => {
            if (next) {
              setState(next);
            }
          }}
        />
      );
    }
    if (activeView === ConsoleView.Skills) {
      return <SkillsView state={state} onRefresh={setState} />;
    }
    if (activeView === ConsoleView.Memory) {
      return <MemoryView />;
    }
    if (activeView === ConsoleView.Knowledge) {
      return <KnowledgeView />;
    }
    if (activeView === ConsoleView.Channels) {
      return <ChannelsView />;
    }
    if (activeView === ConsoleView.Tasks) {
      return <TasksView />;
    }
    if (activeView === ConsoleView.Logs) {
      return <LogsView />;
    }
    if (PLACEHOLDER_CONSOLE_VIEWS.has(activeView)) {
      const pageKey = {
        [ConsoleView.Memory]: "menu_memory",
        [ConsoleView.Knowledge]: "menu_knowledge",
        [ConsoleView.Channels]: "menu_channels",
        [ConsoleView.Tasks]: "menu_tasks",
        [ConsoleView.Logs]: "menu_logs"
      }[activeView];
      return <PlaceholderView viewKey={pageKey} />;
    }
    return null;
  }, [activeView, handleNewSession, setState, state]);

  if (loading) {
    return (
      <TooltipProvider>
        <div className="agent-console flex flex-1 items-center justify-center text-sm text-slate-500">
          {t("loading_state")}
        </div>
      </TooltipProvider>
    );
  }

  if (error) {
    return (
      <TooltipProvider>
        <div className="agent-console flex flex-1 flex-col items-center justify-center gap-2 p-6 text-sm text-red-500">
          <p>{t("load_failed")}</p>
          <p className="text-muted-foreground max-w-md text-center text-xs">{error}</p>
        </div>
      </TooltipProvider>
    );
  }

  return (
    <TooltipProvider>
      <div className="agent-console flex h-full min-h-0 flex-1 overflow-hidden bg-gray-50 font-sans text-slate-800 dark:bg-[#111111] dark:text-slate-200">
        <ConsoleSidebar
          navGroups={sidebarNavGroups}
          activeView={activeView}
          onNavigate={setActiveView}
          openGroups={openGroups}
          onToggleGroup={toggleGroup}
          mobileOpen={mobileSidebarOpen}
          onCloseMobile={() => setMobileSidebarOpen(false)}
        />

        <SessionPanel
          open={sessionPanelOpen}
          sessionId={state?.sessionId}
          onClose={() => setSessionPanelOpen(false)}
          onNewChat={() => void handleNewSession()}
        />

        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          <ConsoleHeader
            activeView={activeView}
            devChannel={devChannel}
            theme={theme}
            onToggleTheme={handleToggleTheme}
            onToggleSessionPanel={() => setSessionPanelOpen((v) => !v)}
            onToggleMobileSidebar={() => setMobileSidebarOpen((v) => !v)}
          />
          <div className="min-h-0 flex-1 overflow-hidden">{viewContent}</div>
        </div>
      </div>
    </TooltipProvider>
  );
}
