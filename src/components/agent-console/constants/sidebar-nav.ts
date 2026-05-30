import { ConsoleView } from "@/enums";
import {
  BookOpen,
  Brain,
  Clock,
  Cpu,
  MessageSquare,
  Radio,
  SlidersHorizontal,
  Terminal,
  Zap,
  type LucideIcon
} from "lucide-react";

export enum SidebarGroupId {
  Chat = "chat",
  Manage = "manage",
  Monitor = "monitor"
}

export interface SidebarNavItem {
  view: ConsoleView;
  icon: LucideIcon;
  labelKey: string;
}

export interface SidebarNavGroup {
  id: SidebarGroupId;
  labelKey: string;
  items: SidebarNavItem[];
}

export const SIDEBAR_NAV_GROUPS: SidebarNavGroup[] = [
  {
    id: SidebarGroupId.Chat,
    labelKey: "nav_chat",
    items: [{ view: ConsoleView.Chat, icon: MessageSquare, labelKey: "menu_chat" }]
  },
  {
    id: SidebarGroupId.Manage,
    labelKey: "nav_manage",
    items: [
      { view: ConsoleView.Config, icon: SlidersHorizontal, labelKey: "menu_config" },
      { view: ConsoleView.Models, icon: Cpu, labelKey: "menu_models" },
      { view: ConsoleView.Skills, icon: Zap, labelKey: "menu_skills" },
      { view: ConsoleView.Memory, icon: Brain, labelKey: "menu_memory" },
      { view: ConsoleView.Knowledge, icon: BookOpen, labelKey: "menu_knowledge" },
      { view: ConsoleView.Channels, icon: Radio, labelKey: "menu_channels" },
      { view: ConsoleView.Tasks, icon: Clock, labelKey: "menu_tasks" }
    ]
  },
  {
    id: SidebarGroupId.Monitor,
    labelKey: "nav_monitor",
    items: [{ view: ConsoleView.Logs, icon: Terminal, labelKey: "menu_logs" }]
  }
];

export const CONSOLE_VIEW_GROUP_LABEL: Partial<Record<ConsoleView, string>> = {
  [ConsoleView.Chat]: "nav_chat",
  [ConsoleView.Config]: "nav_manage",
  [ConsoleView.Models]: "nav_manage",
  [ConsoleView.Skills]: "nav_manage",
  [ConsoleView.Memory]: "nav_manage",
  [ConsoleView.Knowledge]: "nav_manage",
  [ConsoleView.Channels]: "nav_manage",
  [ConsoleView.Tasks]: "nav_manage",
  [ConsoleView.Logs]: "nav_monitor"
};

export const CONSOLE_VIEW_PAGE_LABEL: Record<ConsoleView, string> = {
  [ConsoleView.Chat]: "menu_chat",
  [ConsoleView.Config]: "menu_config",
  [ConsoleView.Models]: "menu_models",
  [ConsoleView.Skills]: "menu_skills",
  [ConsoleView.Memory]: "menu_memory",
  [ConsoleView.Knowledge]: "menu_knowledge",
  [ConsoleView.Channels]: "menu_channels",
  [ConsoleView.Tasks]: "menu_tasks",
  [ConsoleView.Logs]: "menu_logs"
};

export function getBreadcrumbKeys(view: ConsoleView) {
  return {
    groupKey: CONSOLE_VIEW_GROUP_LABEL[view] ?? "nav_chat",
    pageKey: CONSOLE_VIEW_PAGE_LABEL[view]
  };
}

/** Placeholder views not yet wired to Rust IPC. */
export const PLACEHOLDER_CONSOLE_VIEWS = new Set<ConsoleView>([
  ConsoleView.Memory,
  ConsoleView.Knowledge,
  ConsoleView.Channels,
  ConsoleView.Tasks,
  ConsoleView.Logs
]);

export const CONSOLE_BRAND = {
  name: "SupportFlow",
  githubUrl: "https://github.com/kingning2/SupportFlow"
} as const;
