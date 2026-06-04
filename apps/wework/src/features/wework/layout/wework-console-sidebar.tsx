"use client";

import { Building2, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@supportflow/shared";
import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

import { AccountAvatar } from "../accounts/account-avatar";
import { WEWORK_NAV_GROUPS } from "../constants/wework-nav";
import type { WeworkConnectionStatus } from "../types/wework-conversation";

export interface WeworkConsoleSidebarProps {
  activeRoute: WeworkConsoleRoute;
  onNavigate: (route: WeworkConsoleRoute) => void;
  connectionStatus: WeworkConnectionStatus;
  connectedAccountName?: string | null;
  openGroups: Record<string, boolean>;
  onToggleGroup: (groupId: string) => void;
}

function connectionLabelKey(status: WeworkConnectionStatus): string {
  switch (status) {
    case "ready":
      return "wework_status_ready";
    case "connecting":
      return "wework_status_connecting";
    default:
      return "wework_status_disconnected";
  }
}

export function WeworkConsoleSidebar({
  activeRoute,
  onNavigate,
  connectionStatus,
  connectedAccountName,
  openGroups,
  onToggleGroup
}: WeworkConsoleSidebarProps) {
  const { t } = useTranslation("console");
  const showAccount = connectionStatus === "ready" && connectedAccountName;

  return (
    <aside className="wework-console-sidebar flex min-h-0 shrink-0 flex-col">
      <div
        className={cn(
          "shrink-0 border-b border-[hsl(var(--border))] px-4",
          showAccount ? "py-3" : "flex h-14 items-center"
        )}
      >
        {showAccount ? (
          <div className="flex items-center gap-3">
            <AccountAvatar name={connectedAccountName} size="md" />
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-semibold text-[#1A2B4A]">
                {connectedAccountName}
              </p>
              <p className="flex items-center gap-1.5 text-[10px] text-emerald-600">
                <span className="size-1.5 rounded-full bg-emerald-500" />
                {t("wework_status_ready")}
              </p>
            </div>
          </div>
        ) : (
          <div className="flex w-full items-center gap-3">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-[var(--wework-blue)]">
              <Building2 className="size-4 text-white" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-semibold text-[#1A2B4A]">
                {t("wework_brand_title")}
              </p>
              <p className="flex items-center gap-1.5 text-[10px] text-slate-500">
                <span
                  className={cn(
                    "size-1.5 rounded-full",
                    connectionStatus === "connecting" ? "bg-amber-400" : "bg-slate-300"
                  )}
                />
                {t(connectionLabelKey(connectionStatus))}
              </p>
            </div>
          </div>
        )}
      </div>

      <nav className="min-h-0 flex-1 overflow-y-auto px-2 py-3">
        {WEWORK_NAV_GROUPS.map((group) => (
          <div key={group.id} className="mb-2">
            <button
              type="button"
              className="flex w-full cursor-pointer items-center gap-1 px-2 py-1.5 text-[10px] font-semibold tracking-wide text-slate-400 uppercase"
              onClick={() => onToggleGroup(group.id)}
            >
              <ChevronRight
                className={cn("size-3 transition-transform", openGroups[group.id] && "rotate-90")}
              />
              {t(group.labelKey)}
            </button>
            {openGroups[group.id] ? (
              <ul className="mt-0.5 space-y-0.5 pl-1">
                {group.items.map((item) => {
                  const Icon = item.icon;
                  const isActive = activeRoute === item.route;
                  return (
                    <li key={item.route}>
                      <button
                        type="button"
                        className={cn(
                          "wework-console-sidebar-item flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2 text-sm text-slate-600 transition-colors hover:bg-slate-50",
                          isActive && "active"
                        )}
                        onClick={() => onNavigate(item.route)}
                      >
                        <Icon className="size-4 shrink-0 opacity-80" />
                        <span className="truncate">{t(item.labelKey)}</span>
                      </button>
                    </li>
                  );
                })}
              </ul>
            ) : null}
          </div>
        ))}
      </nav>
    </aside>
  );
}
