import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";
import {
  BookOpen,
  Building2,
  MessageSquare,
  Plug,
  SlidersHorizontal,
  Zap,
  type LucideIcon
} from "lucide-react";

export interface WeworkNavItem {
  route: WeworkConsoleRoute;
  icon: LucideIcon;
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
    items: [{ route: WeworkConsoleRoute.Inbox, icon: MessageSquare, label: "对话" }]
  },
  {
    id: "agent",
    label: "智能体",
    items: [
      { route: WeworkConsoleRoute.Knowledge, icon: BookOpen, label: "知识库" },
      { route: WeworkConsoleRoute.Skills, icon: Zap, label: "技能" },
      { route: WeworkConsoleRoute.Mcp, icon: Plug, label: "MCP" },
      {
        route: WeworkConsoleRoute.AiConfig,
        icon: SlidersHorizontal,
        label: "AI 配置"
      }
    ]
  },
  {
    id: "wework",
    label: "企微",
    items: [{ route: WeworkConsoleRoute.Account, icon: Building2, label: "账号与通道" }]
  }
];

export const WEWORK_ROUTE_PAGE_LABEL: Record<WeworkConsoleRoute, string> = {
  [WeworkConsoleRoute.Inbox]: "对话",
  [WeworkConsoleRoute.Knowledge]: "知识库",
  [WeworkConsoleRoute.Skills]: "技能",
  [WeworkConsoleRoute.Mcp]: "MCP",
  [WeworkConsoleRoute.AiConfig]: "AI 配置",
  [WeworkConsoleRoute.Account]: "账号与通道"
};
