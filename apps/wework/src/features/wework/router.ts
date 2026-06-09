"use client";

import { createElement, type ReactNode } from "react";
import { Navigate, type RouteObject } from "react-router-dom";

import type { ChannelCatalogEntry } from "@supportflow/shared";
import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

import { WeworkPage } from "./accounts/wework-page";
import type { WeworkPageActions } from "./accounts/wework-page-types";
import { WEWORK_ROUTE_PAGE_LABEL } from "./constants/wework-nav";
import { InboxView } from "./inbox/inbox-view";
import type { WeworkConnectionStatus } from "./types/wework-conversation";
import { ConfigPlaceholderView } from "./views/config-placeholder-view";
import { KnowledgeView } from "./views/knowledge-view";
import { SkillsView } from "./views/skills-view";

export function toWeworkConsolePath(route: WeworkConsoleRoute): string {
  return `/${route}`;
}

export interface WeworkConsoleRouterDeps {
  fallbackRoute: WeworkConsoleRoute;
  lang: string;
  actions: WeworkPageActions;
  channel: ChannelCatalogEntry | null;
  channelLoading: boolean;
  channelError: string | null;
  connectionStatus: WeworkConnectionStatus;
  onChannelUpdated: () => void;
}

function accountRouteElement(params: WeworkConsoleRouterDeps): ReactNode {
  return createElement(
    "div",
    { className: "min-h-0 flex-1 overflow-y-auto" },
    createElement(WeworkPage, {
      lang: params.lang,
      actions: params.actions,
      channel: params.channel,
      channelLoading: params.channelLoading,
      channelError: params.channelError,
      connectionStatus: params.connectionStatus,
      onChannelUpdated: params.onChannelUpdated
    })
  );
}

export function buildWeworkConsoleRouteObjects(params: WeworkConsoleRouterDeps): RouteObject[] {
  return [
    {
      index: true,
      element: createElement(Navigate, {
        to: toWeworkConsolePath(params.fallbackRoute),
        replace: true
      })
    },
    {
      path: WeworkConsoleRoute.Inbox,
      element: createElement(InboxView, { connectionStatus: params.connectionStatus })
    },
    {
      path: WeworkConsoleRoute.Account,
      element: accountRouteElement(params)
    },
    {
      path: WeworkConsoleRoute.Knowledge,
      element: createElement(KnowledgeView)
    },
    {
      path: WeworkConsoleRoute.Skills,
      element: createElement(SkillsView)
    },
    {
      path: WeworkConsoleRoute.Mcp,
      element: createElement(ConfigPlaceholderView, {
        labelKey: WEWORK_ROUTE_PAGE_LABEL[WeworkConsoleRoute.Mcp]
      })
    },
    {
      path: WeworkConsoleRoute.AiConfig,
      element: createElement(ConfigPlaceholderView, {
        labelKey: WEWORK_ROUTE_PAGE_LABEL[WeworkConsoleRoute.AiConfig]
      })
    },
    {
      path: "*",
      element: createElement(Navigate, {
        to: toWeworkConsolePath(params.fallbackRoute),
        replace: true
      })
    }
  ];
}
