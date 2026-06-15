"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { Banner, Layout } from "@douyinfe/semi-ui-19";
import { Outlet, useLocation, useNavigate } from "react-router-dom";

import {
  isWeworkConsoleRoute,
  LocalCacheKey,
  WeworkConsoleRoute
} from "@supportflow/shared/tauri-bridge/enums";

import { Page } from "./accounts/page";
import type { PageActions } from "./accounts/page-types";
import { useActiveWeworkAccount } from "./hooks/use-active-wework-account";
import { useWeworkChannel } from "./hooks/use-wework-channel";
import { Inbox } from "./inbox/inbox";
import { Header } from "./layout/header";
import { Sidebar } from "./layout/sidebar";
import { WeworkPageSingle, WeworkPageSingleBody } from "./layout/workspace-layout";
import {
  WeworkConsoleContext,
  useWeworkConsoleContext,
  type WeworkConsoleContextValue
} from "./wework-console-context";
import { AiChat } from "./views/ai-chat";
import { AiConfig } from "./views/ai-config";
import { Knowledge } from "./views/knowledge";
import { McpPage } from "./views/mcp";
import { Skills } from "./views/skills";
import { LicenseLockOverlay, LicenseLockedPage } from "@supportflow/ui/license";

const { Content } = Layout;

export interface AppProps {
  lang: string;
  actions: PageActions;
}

const DEFAULT_OPEN_GROUPS: Record<string, boolean> = {
  workspace: true,
  agent: true,
  wework: true
};

export function App({ lang, actions }: AppProps) {
  const { channel, channelLoading, channelError, connectionStatus, refreshChannel } =
    useWeworkChannel(actions);
  const { account: connectedAccount, refreshActiveAccount } =
    useActiveWeworkAccount(connectionStatus);
  const [openGroups, setOpenGroups] = useState(DEFAULT_OPEN_GROUPS);

  const location = useLocation();
  const navigate = useNavigate();

  const activeRoute: WeworkConsoleRoute = useMemo(() => {
    const segments = location.pathname.replace(/^\/+/, "").split("/");
    const last = segments[segments.length - 1] || "";
    return isWeworkConsoleRoute(last) ? last : WeworkConsoleRoute.Inbox;
  }, [location.pathname]);

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
      <Layout className="shell wework-semi-layout h-full min-h-0 flex-1 overflow-hidden">
        <Sidebar
          activeRoute={activeRoute}
          onNavigate={handleNavigate}
          connectionStatus={connectionStatus}
          connectedAccountName={connectedAccount?.label}
          openGroups={openGroups}
          onToggleGroup={toggleGroup}
        />
        <Layout
          style={{
            minHeight: 0,
            minWidth: 0,
            flex: 1,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden"
          }}
        >
          <LicenseLockOverlay enabled={activeRoute === WeworkConsoleRoute.Account}>
            <Layout
              style={{
                height: "100%",
                minHeight: 0,
                minWidth: 0,
                flex: 1,
                display: "flex",
                flexDirection: "column",
                overflow: "hidden",
                background: "var(--main-window-bg)"
              }}
            >
              <Header activeRoute={activeRoute} />
              {connectionStatus !== "ready" && activeRoute === WeworkConsoleRoute.Inbox ? (
                <Banner
                  fullMode={false}
                  bordered
                  type="warning"
                  closeIcon={null}
                  description="请先在「账号与通道」中完成企业微信接入，再使用对话收件箱。"
                />
              ) : null}
              <Content className="wework-semi-content min-h-0 flex-1 overflow-hidden p-0">
                <Outlet />
              </Content>
            </Layout>
          </LicenseLockOverlay>
        </Layout>
      </Layout>
    </WeworkConsoleContext.Provider>
  );
}

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
    <WeworkPageSingle>
      <Page
        lang={lang}
        actions={actions}
        channel={channel}
        channelLoading={channelLoading}
        channelError={channelError}
        connectionStatus={connectionStatus}
        onChannelUpdated={onChannelUpdated}
      />
    </WeworkPageSingle>
  );
}

export function KnowledgeRoute() {
  return <Knowledge />;
}

export function SkillsRoute() {
  return (
    <WeworkPageSingle>
      <WeworkPageSingleBody>
        <Skills />
      </WeworkPageSingleBody>
    </WeworkPageSingle>
  );
}

export function McpRoute() {
  return (
    <WeworkPageSingle>
      <WeworkPageSingleBody>
        <McpPage />
      </WeworkPageSingleBody>
    </WeworkPageSingle>
  );
}

export function AiConfigRoute() {
  return (
    <WeworkPageSingle>
      <WeworkPageSingleBody style={{ padding: 0 }}>
        <AiConfig />
      </WeworkPageSingleBody>
    </WeworkPageSingle>
  );
}

export function AiChatRoute() {
  return (
    <WeworkPageSingle>
      <WeworkPageSingleBody
        style={{ padding: 0, overflow: "hidden", display: "flex", flexDirection: "column" }}
      >
        <AiChat />
      </WeworkPageSingleBody>
    </WeworkPageSingle>
  );
}

export function LicenseRoute() {
  return <LicenseLockedPage />;
}
