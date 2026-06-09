"use client";

import { createElement, type ReactNode } from "react";
import { Navigate, type RouteObject } from "react-router-dom";

import type { ChannelCatalogEntry } from "@supportflow/shared";
import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

import { Page } from "./accounts/page";
import type { PageActions } from "./accounts/page-types";
import { WEWORK_ROUTE_PAGE_LABEL } from "./constants/wework-nav";
import { Inbox } from "./inbox/inbox";
import type { WeworkConnectionStatus } from "./types/wework-conversation";
import { ConfigPlaceholder } from "./views/config-placeholder";
import { Knowledge } from "./views/knowledge";
import { Skills } from "./views/skills";

export function toWeworkConsolePath(route: WeworkConsoleRoute): string {
  return `/${route}`;
}

export interface WeworkConsoleRouterDeps {
  fallbackRoute: WeworkConsoleRoute;
  lang: string;
  actions: PageActions;
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
    createElement(Page, {
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
      element: createElement(Inbox, { connectionStatus: params.connectionStatus })
    },
    {
      path: WeworkConsoleRoute.Account,
      element: accountRouteElement(params)
    },
    {
      path: WeworkConsoleRoute.Knowledge,
      element: createElement(Knowledge)
    },
    {
      path: WeworkConsoleRoute.Skills,
      element: createElement(Skills)
    },
    {
      path: WeworkConsoleRoute.Mcp,
      element: createElement(ConfigPlaceholder, {
        labelKey: WEWORK_ROUTE_PAGE_LABEL[WeworkConsoleRoute.Mcp]
      })
    },
    {
      path: WeworkConsoleRoute.AiConfig,
      element: createElement(ConfigPlaceholder, {
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
