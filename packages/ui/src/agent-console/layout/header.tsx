"use client";

import { ChevronRight, ExternalLink, History, Menu, Moon, Sun } from "lucide-react";

import { CONSOLE_BRAND, getBreadcrumbLabels } from "../constants/sidebar-nav";
import { Button } from "@supportflow/ui/button";
import type { ConsoleView, ChannelCatalogEntryId } from "@supportflow/shared/tauri-bridge/enums";
import type { ConsoleTheme } from "../lib/agent-console/theme-sync";

interface HeaderProps {
  activeView: ConsoleView;
  devChannel: ChannelCatalogEntryId | null;
  theme: ConsoleTheme;
  onToggleTheme: () => void;
  onToggleSessions: () => void;
  onToggleMobileSidebar: () => void;
}

export function Header({
  activeView,
  devChannel,
  theme,
  onToggleTheme,
  onToggleSessions,
  onToggleMobileSidebar
}: HeaderProps) {
  const { groupLabel, pageLabel } = getBreadcrumbLabels(activeView, devChannel);

  return (
    <header className="bg-background border-border z-10 flex h-14 shrink-0 items-center gap-3 border-b px-4">
      <Button
        type="button"
        variant="ghost"
        size="icon-sm"
        className="lg:hidden"
        onClick={onToggleMobileSidebar}
      >
        <Menu className="size-4" />
      </Button>

      <Button type="button" variant="ghost" size="icon-sm" onClick={onToggleSessions}>
        <History className="text-muted-foreground size-4" />
      </Button>

      <div className="hidden min-w-0 items-center gap-2 text-sm lg:flex">
        <span className="text-muted-foreground truncate">{groupLabel}</span>
        <ChevronRight className="text-muted-foreground/60 size-2.5" />
        <span className="text-foreground truncate font-medium">{pageLabel}</span>
      </div>

      <div className="flex-1" />

      <Button type="button" variant="ghost" size="icon-sm" onClick={onToggleTheme}>
        {theme === "dark" ? (
          <Sun className="text-muted-foreground size-4" />
        ) : (
          <Moon className="text-muted-foreground size-4" />
        )}
      </Button>

      <Button type="button" variant="ghost" size="icon-sm" asChild>
        <a href={CONSOLE_BRAND.githubUrl} target="_blank" rel="noopener noreferrer">
          <ExternalLink className="text-muted-foreground size-4" />
        </a>
      </Button>
    </header>
  );
}
