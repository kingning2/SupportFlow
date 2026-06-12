"use client";

import { Collapse, Menu } from "antd";
import type { MenuItemType } from "antd/es/menu/interface";
import { Building2 } from "lucide-react";

import { cn } from "@supportflow/shared";
import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

import { AccountAvatar } from "../accounts/avatar";
import { WEWORK_NAV_GROUPS } from "../constants/wework-nav";
import type { WeworkConnectionStatus } from "../types/wework-conversation";

export interface SidebarProps {
  activeRoute: WeworkConsoleRoute;
  onNavigate: (route: WeworkConsoleRoute) => void;
  connectionStatus: WeworkConnectionStatus;
  connectedAccountName?: string | null;
  openGroups: Record<string, boolean>;
  onToggleGroup: (groupId: string) => void;
}

function connectionLabel(status: WeworkConnectionStatus): string {
  switch (status) {
    case "ready":
      return "已连接";
    case "connecting":
      return "连接中";
    default:
      return "未连接";
  }
}

export function Sidebar({
  activeRoute,
  onNavigate,
  connectionStatus,
  connectedAccountName,
  openGroups,
  onToggleGroup
}: SidebarProps) {
  const showAccount = connectionStatus === "ready" && connectedAccountName;
  const activeGroupKeys = WEWORK_NAV_GROUPS.filter((group) => openGroups[group.id]).map(
    (group) => group.id
  );

  return (
    <aside className="sidebar flex min-h-0 shrink-0 flex-col">
      <div
        className={cn(
          "shrink-0 border-b border-[hsl(var(--border))] px-3",
          showAccount ? "py-2.5" : "flex h-12 items-center"
        )}
      >
        {showAccount ? (
          <div className="flex items-center gap-2.5">
            <AccountAvatar name={connectedAccountName} size="md" />
            <div className="min-w-0 flex-1">
              <p className="text-foreground truncate text-sm font-semibold">
                {connectedAccountName}
              </p>
              <p className="flex items-center gap-1.5 text-[10px] text-emerald-600">
                <span className="size-1.5 rounded-full bg-emerald-500" />
                {"已连接"}
              </p>
            </div>
          </div>
        ) : (
          <div className="flex w-full items-center gap-3">
            <div className="bg-channel flex size-8 shrink-0 items-center justify-center rounded-xl shadow-sm">
              <Building2 className="size-4 text-white" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="text-foreground truncate text-sm font-semibold">{"企微智能客服"}</p>
              <p className="text-muted-foreground flex items-center gap-1.5 text-[10px]">
                <span
                  className={cn(
                    "size-1.5 rounded-full",
                    connectionStatus === "connecting" ? "bg-warning" : "bg-muted-foreground/40"
                  )}
                />
                {connectionLabel(connectionStatus)}
              </p>
            </div>
          </div>
        )}
      </div>

      <nav className="min-h-0 flex-1 overflow-y-auto px-1.5 py-2">
        <Collapse
          ghost
          activeKey={activeGroupKeys}
          onChange={(keys) => {
            const nextKeys = new Set(Array.isArray(keys) ? keys : [keys]);
            for (const group of WEWORK_NAV_GROUPS) {
              const isOpen = openGroups[group.id];
              const nextOpen = nextKeys.has(group.id);
              if (isOpen !== nextOpen) {
                onToggleGroup(group.id);
              }
            }
          }}
          items={WEWORK_NAV_GROUPS.map((group) => ({
            key: group.id,
            label: (
              <span className="text-muted-foreground text-[10px] font-semibold tracking-[0.18em] uppercase">
                {group.label}
              </span>
            ),
            children: (
              <Menu
                mode="inline"
                selectedKeys={[activeRoute]}
                items={group.items.map((item) => {
                  const Icon = item.icon;
                  return {
                    key: item.route,
                    icon: <Icon className="size-4 shrink-0 opacity-80" />,
                    label: <span className="truncate">{item.label}</span>
                  } satisfies MenuItemType;
                })}
                className="sidebar-menu border-none bg-transparent"
                onClick={({ key }) => onNavigate(key as WeworkConsoleRoute)}
              />
            ),
            className: "!mb-1 !border-none !bg-transparent"
          }))}
          className="sidebar-collapse bg-transparent"
        />
      </nav>
    </aside>
  );
}
