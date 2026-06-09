"use client";

import { ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@supportflow/shared";
import type { ConsoleView } from "@supportflow/shared/tauri-bridge/enums";
import { Button } from "@supportflow/ui/button";
import {
  getSidebarNavGroups,
  type SidebarGroupId,
  type SidebarNavGroup
} from "../constants/sidebar-nav";

interface ConsoleSidebarProps {
  navGroups: SidebarNavGroup[];
  activeView: ConsoleView;
  onNavigate: (view: ConsoleView) => void;
  openGroups: Record<SidebarGroupId, boolean>;
  onToggleGroup: (groupId: SidebarGroupId) => void;
  mobileOpen: boolean;
  onCloseMobile: () => void;
}

export function ConsoleSidebar({
  navGroups,
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
          "absolute inset-y-0 left-0 z-50 flex w-52 flex-col bg-[var(--console-sidebar-bg)] text-[hsl(var(--text-tertiary))] transition-transform duration-300 ease-in-out",
          mobileOpen ? "translate-x-0" : "-translate-x-full lg:relative lg:translate-x-0"
        )}
      >
        <div className="border-border/20 flex h-14 shrink-0 items-center gap-3 border-b px-5">
          <div className="bg-primary flex h-8 w-8 shrink-0 items-center justify-center rounded-lg">
            <span className="text-xs font-bold text-white">C</span>
          </div>
          <div className="flex min-w-0 flex-col">
            <span className="truncate text-sm font-semibold text-white">SupportFlow</span>
            <span className="text-xs text-[hsl(var(--text-tertiary))]">{"控制台"}</span>
          </div>
        </div>

        <nav className="flex-1 space-y-1 overflow-y-auto px-3 py-4">
          {navGroups.map((group) => (
            <div
              key={group.id}
              className={cn("menu-group", openGroups[group.id] && "open")}
              data-group={group.id}
            >
              <Button
                type="button"
                variant="ghost"
                className="flex w-full cursor-pointer items-center gap-2 px-3 py-2 text-xs font-semibold tracking-wider text-neutral-500 uppercase transition-colors duration-150 hover:text-neutral-300"
                onClick={() => onToggleGroup(group.id)}
              >
                <ChevronRight className="chevron size-2.5 transition-transform" />
                <span>{t(group.labelKey)}</span>
              </Button>
              <div className="menu-group-items pl-2">
                {group.items.map((item) => {
                  const Icon = item.icon;
                  const isActive = activeView === item.view;
                  return (
                    <Button
                      key={item.view}
                      type="button"
                      variant="ghost"
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
                    </Button>
                  );
                })}
              </div>
            </div>
          ))}
        </nav>

        <div className="border-border/20 shrink-0 border-t px-4 py-3">
          <div className="flex items-center gap-2 text-xs text-[hsl(var(--text-tertiary))]">
            <span className="bg-success size-1.5 rounded-full" />
            <span>SupportFlow Desktop</span>
          </div>
        </div>
      </aside>

      {mobileOpen ? (
        <Button
          type="button"
          aria-label="Close sidebar"
          variant="ghost"
          className="fixed inset-0 z-40 h-auto w-auto rounded-none bg-black/50 lg:hidden"
          onClick={onCloseMobile}
        />
      ) : null}
    </>
  );
}
