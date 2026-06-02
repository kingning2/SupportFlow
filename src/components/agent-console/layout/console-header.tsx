"use client";

import { ChevronRight, ExternalLink, Globe, History, Menu, Moon, Sun } from "lucide-react";
import { useTranslation } from "react-i18next";

import { CONSOLE_BRAND, getBreadcrumbKeys } from "@/components/agent-console/constants/sidebar-nav";
import { Button } from "@/components/ui/button";
import type { ConsoleView, ChannelCatalogEntryId } from "@/enums";
import { Language } from "@/enums";
import type { ConsoleTheme } from "@/lib/agent-console/theme-sync";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { changeCurrentLanguageAction } from "@/store/modules/app";

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
    <header className="z-10 flex h-14 shrink-0 items-center gap-3 border-b border-slate-200 bg-white px-4 dark:border-white/10 dark:bg-[#1A1A1A]">
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
        <History className="size-4 text-slate-500 dark:text-slate-400" />
      </Button>

      <div className="hidden min-w-0 items-center gap-2 text-sm lg:flex">
        <span className="truncate text-slate-400 dark:text-slate-500">{t(groupKey)}</span>
        <ChevronRight className="size-2.5 text-slate-300 dark:text-slate-600" />
        <span className="truncate font-medium text-slate-700 dark:text-slate-200">
          {t(pageKey)}
        </span>
      </div>

      <div className="flex-1" />

      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="gap-1.5 text-slate-500 dark:text-slate-400"
        onClick={toggleLanguage}
      >
        <Globe className="size-3.5" />
        <span>{currentLanguage === Language.Cn ? t("switch_to_en") : t("switch_to_cn")}</span>
      </Button>

      <Button type="button" variant="ghost" size="icon-sm" onClick={onToggleTheme}>
        {theme === "dark" ? (
          <Sun className="size-4 text-slate-500" />
        ) : (
          <Moon className="size-4 text-slate-500" />
        )}
      </Button>

      <Button type="button" variant="ghost" size="icon-sm" asChild>
        <a href={CONSOLE_BRAND.githubUrl} target="_blank" rel="noopener noreferrer">
          <ExternalLink className="size-4 text-slate-500" />
        </a>
      </Button>
    </header>
  );
}
