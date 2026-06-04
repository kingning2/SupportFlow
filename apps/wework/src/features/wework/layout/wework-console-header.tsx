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
    <header className="flex h-12 shrink-0 items-center border-b border-[hsl(var(--border))] bg-white px-4">
      <h1 className="text-sm font-semibold text-[#1A2B4A]">{t(pageKey)}</h1>
    </header>
  );
}
