"use client";

import { ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";
import type { ConsoleView } from "@/enums";
import {
  SIDEBAR_NAV_GROUPS,
  type SidebarGroupId
} from "@/components/agent-console/constants/sidebar-nav";

interface ConsoleSidebarProps {
  activeView: ConsoleView;
  onNavigate: (view: ConsoleView) => void;
  openGroups: Record<SidebarGroupId, boolean>;
  onToggleGroup: (groupId: SidebarGroupId) => void;
  mobileOpen: boolean;
  onCloseMobile: () => void;
}

export function ConsoleSidebar({
  activeView,
  onNavigate,
  openGroups,
  onToggleGroup,
  mobileOpen,
  onCloseMobile
}: ConsoleSidebarProps) {
  const { t } = useTranslation("console");

  return (
    <>
      <aside
        className={cn(
          "fixed inset-y-0 left-0 z-50 flex w-52 flex-col bg-[#0A0A0A] text-neutral-400 transition-transform duration-300 ease-in-out",
          mobileOpen ? "translate-x-0" : "-translate-x-full lg:relative lg:translate-x-0"
        )}
      >
        <div className="flex h-14 shrink-0 items-center gap-3 border-b border-white/10 px-5">
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-[#35A85B]">
            <span className="text-xs font-bold text-white">C</span>
          </div>
          <div className="flex min-w-0 flex-col">
            <span className="truncate text-sm font-semibold text-white">CowAgent</span>
            <span className="text-xs text-neutral-500">{t("console")}</span>
          </div>
        </div>

        <nav className="flex-1 space-y-1 overflow-y-auto px-3 py-4">
          {SIDEBAR_NAV_GROUPS.map((group) => (
            <div
              key={group.id}
              className={cn("menu-group", openGroups[group.id] && "open")}
              data-group={group.id}
            >
              <button
                type="button"
                className="flex w-full cursor-pointer items-center gap-2 px-3 py-2 text-xs font-semibold tracking-wider text-neutral-500 uppercase transition-colors duration-150 hover:text-neutral-300"
                onClick={() => onToggleGroup(group.id)}
              >
                <ChevronRight className="chevron size-2.5 transition-transform" />
                <span>{t(group.labelKey)}</span>
              </button>
              <div className="menu-group-items pl-2">
                {group.items.map((item) => {
                  const Icon = item.icon;
                  const isActive = activeView === item.view;
                  return (
                    <button
                      key={item.view}
                      type="button"
                      className={cn(
                        "sidebar-item flex w-full cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-[14px] transition-all duration-150 hover:bg-white/5 hover:text-neutral-200",
                        isActive && "active"
                      )}
                      onClick={() => {
                        onNavigate(item.view);
                        onCloseMobile();
                      }}
                    >
                      <Icon className="item-icon size-4 shrink-0" />
                      <span>{t(item.labelKey)}</span>
                    </button>
                  );
                })}
              </div>
            </div>
          ))}
        </nav>

        <div className="shrink-0 border-t border-white/10 px-4 py-3">
          <div className="flex items-center gap-2 text-xs text-neutral-600">
            <span className="size-1.5 rounded-full bg-[#4ABE6E]" />
            <span>CowAgent Desktop</span>
          </div>
        </div>
      </aside>

      {mobileOpen ? (
        <button
          type="button"
          aria-label="Close sidebar"
          className="fixed inset-0 z-40 bg-black/50 lg:hidden"
          onClick={onCloseMobile}
        />
      ) : null}
    </>
  );
}
