"use client";

import { ChevronRight, ExternalLink, Globe, History, Menu, Moon, Sun } from "lucide-react";
import { useTranslation } from "react-i18next";

import { CONSOLE_BRAND, getBreadcrumbKeys } from "../constants/sidebar-nav";
import { Button } from "@supportflow/ui/button";
import type { ConsoleView, ChannelCatalogEntryId } from "@supportflow/shared/tauri-bridge/enums";
import { Language } from "@supportflow/shared/tauri-bridge/enums";
import type { ConsoleTheme } from "../lib/agent-console/theme-sync";
import { useAppDispatch, useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";
import { changeCurrentLanguageAction } from "@supportflow/shared/desktop-shell/store/modules/app";

interface ConsoleHeaderProps {
  activeView: ConsoleView;
  devChannel: ChannelCatalogEntryId | null;
  theme: ConsoleTheme;
  onToggleTheme: () => void;
  onToggleSessionPanel: () => void;
  onToggleMobileSidebar: () => void;
}

export function ConsoleHeader({
  activeView,
  devChannel,
  theme,
  onToggleTheme,
  onToggleSessionPanel,
  onToggleMobileSidebar
}: ConsoleHeaderProps) {
  const { t, i18n } = useTranslation("console");
  const dispatch = useAppDispatch();
  const currentLanguage = useAppSelector((state) => state.app.currentLanguage);
  const { groupKey, pageKey } = getBreadcrumbKeys(activeView, devChannel);

  const toggleLanguage = () => {
    const next = currentLanguage === Language.Cn ? Language.En : Language.Cn;
    dispatch(changeCurrentLanguageAction(next));
    void i18n.changeLanguage(next);
  };

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

      <Button type="button" variant="ghost" size="icon-sm" onClick={onToggleSessionPanel}>
        <History className="text-muted-foreground size-4" />
      </Button>

      <div className="hidden min-w-0 items-center gap-2 text-sm lg:flex">
        <span className="text-muted-foreground truncate">{t(groupKey)}</span>
        <ChevronRight className="text-muted-foreground/60 size-2.5" />
        <span className="text-foreground truncate font-medium">{t(pageKey)}</span>
      </div>

      <div className="flex-1" />

      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="text-muted-foreground gap-1.5"
        onClick={toggleLanguage}
      >
        <Globe className="size-3.5" />
        <span>{currentLanguage === Language.Cn ? t("switch_to_en") : t("switch_to_cn")}</span>
      </Button>

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
