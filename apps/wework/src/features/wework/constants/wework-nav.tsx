import {
  IconApartment,
  IconBolt,
  IconBookStroked,
  IconCommentStroked,
  IconConfigStroked,
  IconCustomerSupport,
  IconServerStroked
} from "@douyinfe/semi-icons";
import type { ReactNode } from "react";

import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

export interface WeworkNavItem {
  route: WeworkConsoleRoute;
  icon: ReactNode;
  label: string;
}

export interface WeworkNavGroup {
  id: "workspace" | "agent" | "wework";
  label: string;
  items: WeworkNavItem[];
}

export const WEWORK_NAV_GROUPS: WeworkNavGroup[] = [
  {
    id: "workspace",
    label: "工作台",
    items: [{ route: WeworkConsoleRoute.Inbox, icon: <IconCommentStroked />, label: "对话" }]
  },
  {
    id: "agent",
    label: "智能体",
    items: [
      { route: WeworkConsoleRoute.Knowledge, icon: <IconBookStroked />, label: "知识库" },
      { route: WeworkConsoleRoute.Skills, icon: <IconBolt />, label: "技能" },
      { route: WeworkConsoleRoute.AiChat, icon: <IconCustomerSupport />, label: "AI 助手" },
      { route: WeworkConsoleRoute.Mcp, icon: <IconServerStroked />, label: "MCP" },
      {
        route: WeworkConsoleRoute.AiConfig,
        icon: <IconConfigStroked />,
        label: "AI 配置"
      }
    ]
  },
  {
    id: "wework",
    label: "企微",
    items: [{ route: WeworkConsoleRoute.Account, icon: <IconApartment />, label: "账号与通道" }]
  }
];

export const WEWORK_ROUTE_PAGE_LABEL: Record<WeworkConsoleRoute, string> = {
  [WeworkConsoleRoute.Inbox]: "对话",
  [WeworkConsoleRoute.Knowledge]: "知识库",
  [WeworkConsoleRoute.Skills]: "技能",
  [WeworkConsoleRoute.AiChat]: "AI 助手",
  [WeworkConsoleRoute.Mcp]: "MCP",
  [WeworkConsoleRoute.AiConfig]: "AI 配置",
  [WeworkConsoleRoute.Account]: "账号与通道"
};
