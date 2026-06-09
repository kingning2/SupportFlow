"use client";

import { useCallback, useMemo, useState } from "react";

import {
  isWeworkConsoleRoute,
  LocalCacheKey,
  WeworkConsoleRoute
} from "@supportflow/shared/tauri-bridge/enums";

import { WeworkPage } from "@/features/wework/accounts/wework-page";
import type { WeworkPageActions } from "@/features/wework/accounts/wework-page-types";

import { WEWORK_ROUTE_PAGE_LABEL } from "./constants/wework-nav";
import { useActiveWeworkAccount } from "./hooks/use-active-wework-account";
import { useWeworkChannel } from "./hooks/use-wework-channel";
import { InboxView } from "./inbox/inbox-view";
import { WeworkConsoleHeader } from "./layout/wework-console-header";
import { WeworkConsoleSidebar } from "./layout/wework-console-sidebar";
import { ConfigPlaceholderView } from "./views/config-placeholder-view";
import { KnowledgeView } from "./views/knowledge-view";
import { SkillsView } from "./views/skills-view";

const DEFAULT_OPEN_GROUPS: Record<string, boolean> = {
  workspace: true,
  agent: true,
  wework: true
};

function readStoredRoute(): WeworkConsoleRoute {
  if (typeof window === "undefined") {
    return WeworkConsoleRoute.Inbox;
  }
  const raw = localStorage.getItem(LocalCacheKey.WeworkConsoleRoute);
  return raw && isWeworkConsoleRoute(raw) ? raw : WeworkConsoleRoute.Inbox;
}

export interface WeworkConsoleAppProps {
  lang: string;
  actions: WeworkPageActions;
}

export function WeworkConsoleApp({ lang, actions }: WeworkConsoleAppProps) {
  const [activeRoute, setActiveRoute] = useState<WeworkConsoleRoute>(readStoredRoute);
  const [openGroups, setOpenGroups] = useState(DEFAULT_OPEN_GROUPS);

  const { channel, channelLoading, channelError, connectionStatus, refreshChannel } =
    useWeworkChannel(actions);
  const { account: connectedAccount, refreshActiveAccount } =
    useActiveWeworkAccount(connectionStatus);

  const handleChannelUpdated = useCallback(() => {
    void refreshChannel({ silent: true });
    refreshActiveAccount();
  }, [refreshChannel, refreshActiveAccount]);

  const navigate = useCallback((route: WeworkConsoleRoute) => {
    setActiveRoute(route);
    localStorage.setItem(LocalCacheKey.WeworkConsoleRoute, route);
  }, []);

  const toggleGroup = useCallback((groupId: string) => {
    setOpenGroups((prev) => ({ ...prev, [groupId]: !prev[groupId] }));
  }, []);

  const mainContent = useMemo(() => {
    if (activeRoute === WeworkConsoleRoute.Inbox) {
      return <InboxView connectionStatus={connectionStatus} />;
    }
    if (activeRoute === WeworkConsoleRoute.Account) {
      return (
        <div className="min-h-0 flex-1 overflow-y-auto">
          <WeworkPage
            lang={lang}
            actions={actions}
            channel={channel}
            channelLoading={channelLoading}
            channelError={channelError}
            connectionStatus={connectionStatus}
            onChannelUpdated={handleChannelUpdated}
          />
        </div>
      );
    }
    if (activeRoute === WeworkConsoleRoute.Knowledge) {
      // Wework-specific knowledge view using antd components for light enterprise layout,
      // sharing the same IPC upload/list/read/graph with markitdown backend.
      return <KnowledgeView />;
    }
    if (activeRoute === WeworkConsoleRoute.Skills) {
      return <SkillsView />;
    }
    const pageKey = WEWORK_ROUTE_PAGE_LABEL[activeRoute];
    return <ConfigPlaceholderView labelKey={pageKey} />;
  }, [activeRoute, actions, connectionStatus, handleChannelUpdated, lang]);

  return (
    <div className="wework-console flex h-full min-h-0 flex-1 overflow-hidden">
      <WeworkConsoleSidebar
        activeRoute={activeRoute}
        onNavigate={navigate}
        connectionStatus={connectionStatus}
        connectedAccountName={connectedAccount?.label}
        openGroups={openGroups}
        onToggleGroup={toggleGroup}
      />
      <div className="flex min-h-0 min-w-0 flex-1 flex-col bg-[var(--wework-canvas)]">
        <WeworkConsoleHeader activeRoute={activeRoute} />
        {connectionStatus !== "ready" && activeRoute === WeworkConsoleRoute.Inbox ? (
          <div className="bg-warning/10 border-warning/20 mx-3 mt-3 shrink-0 rounded-xl border px-3 py-2 text-xs">
            {"请先在「账号与通道」完成企微接入，再使用对话收件箱。"}
          </div>
        ) : null}
        <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">{mainContent}</div>
      </div>
    </div>
  );
}
