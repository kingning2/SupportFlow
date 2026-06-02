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
  labelKey: string;
}

export interface WeworkNavGroup {
  id: "workspace" | "agent" | "wework";
  labelKey: string;
  items: WeworkNavItem[];
}

export const WEWORK_NAV_GROUPS: WeworkNavGroup[] = [
  {
    id: "workspace",
    labelKey: "wework_nav_workspace",
    items: [{ route: WeworkConsoleRoute.Inbox, icon: MessageSquare, labelKey: "wework_menu_inbox" }]
  },
  {
    id: "agent",
    labelKey: "wework_nav_agent",
    items: [
      { route: WeworkConsoleRoute.Knowledge, icon: BookOpen, labelKey: "menu_knowledge" },
      { route: WeworkConsoleRoute.Skills, icon: Zap, labelKey: "menu_skills" },
      { route: WeworkConsoleRoute.Mcp, icon: Plug, labelKey: "wework_menu_mcp" },
      {
        route: WeworkConsoleRoute.AiConfig,
        icon: SlidersHorizontal,
        labelKey: "wework_menu_ai_config"
      }
    ]
  },
  {
    id: "wework",
    labelKey: "wework_nav_channel",
    items: [{ route: WeworkConsoleRoute.Account, icon: Building2, labelKey: "wework_menu_account" }]
  }
];

export const WEWORK_ROUTE_PAGE_LABEL: Record<WeworkConsoleRoute, string> = {
  [WeworkConsoleRoute.Inbox]: "wework_menu_inbox",
  [WeworkConsoleRoute.Knowledge]: "menu_knowledge",
  [WeworkConsoleRoute.Skills]: "menu_skills",
  [WeworkConsoleRoute.Mcp]: "wework_menu_mcp",
  [WeworkConsoleRoute.AiConfig]: "wework_menu_ai_config",
  [WeworkConsoleRoute.Account]: "wework_menu_account"
};
