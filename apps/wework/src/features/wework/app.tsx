"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode
} from "react";
import { Outlet, useLocation, useNavigate } from "react-router-dom";

import type { ChannelCatalogEntry } from "@supportflow/shared";
import {
  isWeworkConsoleRoute,
  LocalCacheKey,
  WeworkConsoleRoute
} from "@supportflow/shared/tauri-bridge/enums";

import { Page } from "./accounts/page";
import type { PageActions } from "./accounts/page-types";
import { WEWORK_ROUTE_PAGE_LABEL } from "./constants/wework-nav";
import { useActiveWeworkAccount } from "./hooks/use-active-wework-account";
import { useWeworkChannel } from "./hooks/use-wework-channel";
import { Inbox } from "./inbox/inbox";
import { Header } from "./layout/header";
import { Sidebar } from "./layout/sidebar";
import type { WeworkConnectionStatus } from "./types/wework-conversation";
import { ConfigPlaceholder } from "./views/config-placeholder";
import { Knowledge } from "./views/knowledge";
import { Skills } from "./views/skills";

// ── Context ──────────────────────────────────────────────
export interface WeworkConsoleContextValue {
  lang: string;
  actions: PageActions;
  channel: ChannelCatalogEntry | null;
  channelLoading: boolean;
  channelError: string | null;
  connectionStatus: WeworkConnectionStatus;
  onChannelUpdated: () => void;
}

const WeworkConsoleContext = createContext<WeworkConsoleContextValue | null>(null);

export function useWeworkConsoleContext(): WeworkConsoleContextValue {
  const ctx = useContext(WeworkConsoleContext);
  if (!ctx) {
    throw new Error("useWeworkConsoleContext must be used within <WeworkConsoleLayout />");
  }
  return ctx;
}

// ── Layout ───────────────────────────────────────────────
export interface AppProps {
  lang: string;
  actions: PageActions;
}

const DEFAULT_OPEN_GROUPS: Record<string, boolean> = {
  workspace: true,
  agent: true,
  wework: true
};

/** 企微控制台壳布局：侧边栏 + 顶部标题 + Outlet 渲染子路由 */
export function App({ lang, actions }: AppProps) {
  const { channel, channelLoading, channelError, connectionStatus, refreshChannel } =
    useWeworkChannel(actions);
  const { account: connectedAccount, refreshActiveAccount } =
    useActiveWeworkAccount(connectionStatus);
  const [openGroups, setOpenGroups] = useState(DEFAULT_OPEN_GROUPS);

  const location = useLocation();
  const navigate = useNavigate();

  // 从 URL pathname 推导当前激活路由
  const activeRoute: WeworkConsoleRoute = useMemo(() => {
    const segments = location.pathname.replace(/^\/+/, "").split("/");
    const last = segments[segments.length - 1] || "";
    return isWeworkConsoleRoute(last) ? last : WeworkConsoleRoute.Inbox;
  }, [location.pathname]);

  // 持久化到 localStorage 以便 session 恢复
  useEffect(() => {
    localStorage.setItem(LocalCacheKey.WeworkConsoleRoute, activeRoute);
  }, [activeRoute]);

  const handleChannelUpdated = useCallback(() => {
    void refreshChannel({ silent: true });
    refreshActiveAccount();
  }, [refreshChannel, refreshActiveAccount]);

  const handleNavigate = useCallback(
    (route: WeworkConsoleRoute) => {
      navigate(`/${route}`);
    },
    [navigate]
  );

  const toggleGroup = useCallback((groupId: string) => {
    setOpenGroups((prev) => ({ ...prev, [groupId]: !prev[groupId] }));
  }, []);

  const ctxValue = useMemo<WeworkConsoleContextValue>(
    () => ({
      lang,
      actions,
      channel,
      channelLoading,
      channelError,
      connectionStatus,
      onChannelUpdated: handleChannelUpdated
    }),
    [lang, actions, channel, channelLoading, channelError, connectionStatus, handleChannelUpdated]
  );

  return (
    <WeworkConsoleContext.Provider value={ctxValue}>
      <div className="shell flex h-full min-h-0 flex-1 overflow-hidden">
        <Sidebar
          activeRoute={activeRoute}
          onNavigate={handleNavigate}
          connectionStatus={connectionStatus}
          connectedAccountName={connectedAccount?.label}
          openGroups={openGroups}
          onToggleGroup={toggleGroup}
        />
        <div className="flex min-h-0 min-w-0 flex-1 flex-col bg-[var(--wework-canvas)]">
          <Header activeRoute={activeRoute} />
          {connectionStatus !== "ready" && activeRoute === WeworkConsoleRoute.Inbox ? (
            <div className="bg-warning/10 border-warning/20 mx-3 mt-3 shrink-0 rounded-xl border px-3 py-2 text-xs">
              {"请先在「账号与通道」完成企微接入，再使用对话收件箱。"}
            </div>
          ) : null}
          <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
            <Outlet />
          </div>
        </div>
      </div>
    </WeworkConsoleContext.Provider>
  );
}

// ── 子路由组件（消费 Context） ──────────────────────────────

export function InboxRoute() {
  const { connectionStatus } = useWeworkConsoleContext();
  return <Inbox connectionStatus={connectionStatus} />;
}

export function AccountRoute() {
  const {
    lang,
    actions,
    channel,
    channelLoading,
    channelError,
    connectionStatus,
    onChannelUpdated
  } = useWeworkConsoleContext();
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <Page
        lang={lang}
        actions={actions}
        channel={channel}
        channelLoading={channelLoading}
        channelError={channelError}
        connectionStatus={connectionStatus}
        onChannelUpdated={onChannelUpdated}
      />
    </div>
  );
}

export function KnowledgeRoute() {
  return <Knowledge />;
}

export function SkillsRoute() {
  return <Skills />;
}

export function McpRoute() {
  return <ConfigPlaceholder labelKey={WEWORK_ROUTE_PAGE_LABEL[WeworkConsoleRoute.Mcp]} />;
}

export function AiConfigRoute() {
  return <ConfigPlaceholder labelKey={WEWORK_ROUTE_PAGE_LABEL[WeworkConsoleRoute.AiConfig]} />;
}
