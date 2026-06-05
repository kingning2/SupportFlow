"use client";

import { useTranslation } from "react-i18next";

import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

import { WEWORK_ROUTE_PAGE_LABEL } from "../constants/wework-nav";

export interface WeworkConsoleHeaderProps {
  activeRoute: WeworkConsoleRoute;
}

export function WeworkConsoleHeader({ activeRoute }: WeworkConsoleHeaderProps) {
  const { t } = useTranslation("console");
  const pageKey = WEWORK_ROUTE_PAGE_LABEL[activeRoute];

  if (activeRoute === WeworkConsoleRoute.Inbox) {
    return null;
  }

  return (
    <header className="bg-card/80 border-border/60 mx-3 mt-3 flex h-11 shrink-0 items-center rounded-xl border px-4 shadow-sm backdrop-blur">
      <h1 className="text-foreground text-sm font-semibold">{t(pageKey)}</h1>
    </header>
  );
}
