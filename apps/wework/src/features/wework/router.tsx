"use client";

import type { RouteObject } from "react-router-dom";

import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

import {
  AccountRoute,
  AiConfigRoute,
  InboxRoute,
  KnowledgeRoute,
  McpRoute,
  SkillsRoute
} from "./app";

/**
 * 将 WeworkConsoleRoute 转为 URL 路径。
 * 示例：WeworkConsoleRoute.Knowledge → "/knowledge"
 */
export function toWeworkConsolePath(route: WeworkConsoleRoute): string {
  return `/${route}`;
}

/** 创建企微控制台二级路由配置，供 createBrowserRouter 使用。 */
export function buildWeworkConsoleRouteObjects(): RouteObject[] {
  return [
    { index: true, element: <InboxRoute /> },
    { path: WeworkConsoleRoute.Inbox, element: <InboxRoute /> },
    { path: WeworkConsoleRoute.Knowledge, element: <KnowledgeRoute /> },
    { path: WeworkConsoleRoute.Skills, element: <SkillsRoute /> },
    { path: WeworkConsoleRoute.Account, element: <AccountRoute /> },
    { path: WeworkConsoleRoute.Mcp, element: <McpRoute /> },
    { path: WeworkConsoleRoute.AiConfig, element: <AiConfigRoute /> }
  ];
}
