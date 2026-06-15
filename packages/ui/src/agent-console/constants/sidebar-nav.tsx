import type { ReactNode } from "react";
import {
  IconBolt,
  IconBookStroked,
  IconBookmark,
  IconClock,
  IconCommentStroked,
  IconConfigStroked,
  IconConnectionPoint2,
  IconServer,
  IconTerminal
} from "@douyinfe/semi-icons";

import {
  ConsoleView,
  channelLabel,
  type ChannelCatalogEntryId
} from "@supportflow/shared/tauri-bridge/enums";

export enum SidebarGroupId {
  Chat = "chat",
  Manage = "manage",
  Monitor = "monitor"
}

export interface SidebarNavItem {
  view: ConsoleView;
  icon: ReactNode;
  label: string;
}

export interface SidebarNavGroup {
  id: SidebarGroupId;
  label: string;
  items: SidebarNavItem[];
}

function channelNavItem(devChannel: ChannelCatalogEntryId): SidebarNavItem {
  return {
    view: ConsoleView.Channels,
    icon: <IconConnectionPoint2 />,
    label: channelLabel(devChannel)
  };
}

export function getSidebarNavGroups(devChannel: ChannelCatalogEntryId | null): SidebarNavGroup[] {
  const channelItem: SidebarNavItem = devChannel
    ? channelNavItem(devChannel)
    : { view: ConsoleView.Channels, icon: <IconConnectionPoint2 />, label: "通道" };

  return [
    {
      id: SidebarGroupId.Chat,
      label: "对话",
      items: [{ view: ConsoleView.Chat, icon: <IconCommentStroked />, label: "聊天" }]
    },
    {
      id: SidebarGroupId.Manage,
      label: "管理",
      items: [
        { view: ConsoleView.Config, icon: <IconConfigStroked />, label: "配置" },
        { view: ConsoleView.Models, icon: <IconServer />, label: "模型" },
        { view: ConsoleView.Skills, icon: <IconBolt />, label: "技能" },
        { view: ConsoleView.Memory, icon: <IconBookmark />, label: "记忆" },
        { view: ConsoleView.Knowledge, icon: <IconBookStroked />, label: "知识库" },
        channelItem,
        { view: ConsoleView.Tasks, icon: <IconClock />, label: "任务" }
      ]
    },
    {
      id: SidebarGroupId.Monitor,
      label: "监控",
      items: [{ view: ConsoleView.Logs, icon: <IconTerminal />, label: "日志" }]
    }
  ];
}

export const SIDEBAR_NAV_GROUPS = getSidebarNavGroups(null);

export const CONSOLE_VIEW_GROUP_LABEL: Partial<Record<ConsoleView, string>> = {
  [ConsoleView.Chat]: "对话",
  [ConsoleView.Config]: "管理",
  [ConsoleView.Models]: "管理",
  [ConsoleView.Skills]: "管理",
  [ConsoleView.Memory]: "管理",
  [ConsoleView.Knowledge]: "管理",
  [ConsoleView.Channels]: "管理",
  [ConsoleView.Tasks]: "管理",
  [ConsoleView.Logs]: "监控"
};

export const CONSOLE_VIEW_PAGE_LABEL: Record<ConsoleView, string> = {
  [ConsoleView.Chat]: "聊天",
  [ConsoleView.Config]: "配置",
  [ConsoleView.Models]: "模型",
  [ConsoleView.Skills]: "技能",
  [ConsoleView.Memory]: "记忆",
  [ConsoleView.Knowledge]: "知识库",
  [ConsoleView.Channels]: "通道",
  [ConsoleView.Tasks]: "任务",
  [ConsoleView.Logs]: "日志"
};

export function getBreadcrumbLabels(
  view: ConsoleView,
  devChannel: ChannelCatalogEntryId | null = null
) {
  const pageLabel =
    view === ConsoleView.Channels && devChannel
      ? channelLabel(devChannel)
      : CONSOLE_VIEW_PAGE_LABEL[view];
  return {
    groupLabel: CONSOLE_VIEW_GROUP_LABEL[view] ?? "对话",
    pageLabel
  };
}

export const PLACEHOLDER_CONSOLE_VIEWS = new Set<ConsoleView>([]);

export const CONSOLE_BRAND = {
  name: "SupportFlow",
  githubUrl: "https://github.com/kingning2/SupportFlow"
} as const;
