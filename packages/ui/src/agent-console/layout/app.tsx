"use client";

import { useCallback, useMemo, useState } from "react";

import { Chat } from "../chat/chat";
import {
  getSidebarNavGroups,
  PLACEHOLDER_CONSOLE_VIEWS,
  SidebarGroupId
} from "../constants/sidebar-nav";
import { Header } from "../layout/header";
import { Sidebar } from "../layout/sidebar";
import { Sessions } from "../layout/sessions";
import { PlaceholderView } from "../shared/console-brand";
import { Channels } from "../views/channels";
import { Config } from "../views/config";
import { Knowledge } from "../views/knowledge";
import { Logs } from "../views/logs";
import { Memory } from "../views/memory";
import { Models } from "../views/models";
import { Skills } from "../views/skills";
import { Tasks } from "../views/tasks";
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
  const { state, setState, loading, error } = useAgentConsoleState();
  const devChannel = useMemo(() => getDevChannel(), []);
  const sidebarNavGroups = useMemo(() => getSidebarNavGroups(devChannel), [devChannel]);

  const [activeView, setActiveView] = useState<ConsoleView>(ConsoleView.Chat);
  const [theme, setTheme] = useState<ConsoleTheme>(() => {
    const initial = readConsoleTheme();
    applyConsoleTheme(initial);
    return initial;
  });
  const [sessionsOpen, setSessionsOpen] = useState(false);
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
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
        <Chat
          sessionId={state?.sessionId}
          consoleState={state}
          onNewSession={() => void handleNewSession()}
        />
      );
    }
    if (activeView === ConsoleView.Config) {
      return <Config state={state} />;
    }
    if (activeView === ConsoleView.Models) {
      return (
        <Models
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
      return <Skills state={state} onRefresh={setState} />;
    }
    if (activeView === ConsoleView.Memory) {
      return <Memory />;
    }
    if (activeView === ConsoleView.Knowledge) {
      return <Knowledge />;
    }
    if (activeView === ConsoleView.Channels) {
      return <Channels />;
    }
    if (activeView === ConsoleView.Tasks) {
      return <Tasks />;
    }
    if (activeView === ConsoleView.Logs) {
      return <Logs />;
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
          {"正在加载 Agent…"}
        </div>
      </TooltipProvider>
    );
  }

  if (error) {
    return (
      <TooltipProvider>
        <div className="agent-console flex flex-1 flex-col items-center justify-center gap-2 p-6 text-sm text-red-500">
          <p>{"无法初始化 Agent"}</p>
          <p className="text-muted-foreground max-w-md text-center text-xs">{error}</p>
        </div>
      </TooltipProvider>
    );
  }

  return (
    <TooltipProvider>
      <div className="agent-console relative flex h-full min-h-0 flex-1 overflow-hidden bg-gray-50 font-sans text-slate-800 dark:bg-[#111111] dark:text-slate-200">
        <Sidebar
          navGroups={sidebarNavGroups}
          activeView={activeView}
          onNavigate={setActiveView}
          openGroups={openGroups}
          onToggleGroup={toggleGroup}
          mobileOpen={mobileNavOpen}
          onCloseMobile={() => setMobileNavOpen(false)}
        />

        <Sessions
          open={sessionsOpen}
          sessionId={state?.sessionId}
          onClose={() => setSessionsOpen(false)}
          onNewChat={() => void handleNewSession()}
        />

        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          <Header
            activeView={activeView}
            devChannel={devChannel}
            theme={theme}
            onToggleTheme={handleToggleTheme}
            onToggleSessions={() => setSessionsOpen((v) => !v)}
            onToggleMobileSidebar={() => setMobileNavOpen((v) => !v)}
          />
          <div className="min-h-0 flex-1 overflow-hidden">{viewContent}</div>
        </div>
      </div>
    </TooltipProvider>
  );
}
