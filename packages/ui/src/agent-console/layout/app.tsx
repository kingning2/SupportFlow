"use client";

import { useCallback, useMemo, useState } from "react";
import { Empty, Layout, Spin, Typography } from "@douyinfe/semi-ui-19";

import { Chat } from "../chat/chat";
import {
  CONSOLE_VIEW_PAGE_LABEL,
  getSidebarNavGroups,
  PLACEHOLDER_CONSOLE_VIEWS,
  SidebarGroupId
} from "../constants/sidebar-nav";
import { ConsoleHeader } from "../layout/header";
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

const { Content } = Layout;
const { Text } = Typography;

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
        <div className="agent-chat-page">
          <Chat
            sessionId={state?.sessionId}
            consoleState={state}
            onNewSession={() => void handleNewSession()}
          />
        </div>
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
      return <PlaceholderView title={CONSOLE_VIEW_PAGE_LABEL[activeView]} />;
    }
    return null;
  }, [activeView, handleNewSession, setState, state]);

  if (loading) {
    return (
      <Layout
        className="agent-console"
        style={{ flex: 1, alignItems: "center", justifyContent: "center" }}
      >
        <Spin tip="正在加载 Agent..." />
      </Layout>
    );
  }

  if (error) {
    return (
      <Layout
        className="agent-console"
        style={{ flex: 1, alignItems: "center", justifyContent: "center", padding: 24 }}
      >
        <Empty title="无法初始化 Agent" description={error} />
      </Layout>
    );
  }

  return (
    <Layout
      className="agent-console"
      style={{ position: "relative", flex: 1, minHeight: 0, overflow: "hidden" }}
    >
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

      <Layout
        style={{ minHeight: 0, minWidth: 0, flex: 1, display: "flex", flexDirection: "column" }}
      >
        <ConsoleHeader
          activeView={activeView}
          devChannel={devChannel}
          theme={theme}
          onToggleTheme={handleToggleTheme}
          onToggleSessions={() => setSessionsOpen((v) => !v)}
          onToggleMobileSidebar={() => setMobileNavOpen((v) => !v)}
        />
        <Content style={{ minHeight: 0, flex: 1, overflow: "hidden" }}>{viewContent}</Content>
      </Layout>
    </Layout>
  );
}
