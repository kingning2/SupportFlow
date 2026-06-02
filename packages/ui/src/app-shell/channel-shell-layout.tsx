"use client";

import { memo, type ReactNode } from "react";

import { ChannelTitleBar, type ChannelTitleBarAccent } from "@supportflow/ui/title-bar";

import { useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";

export type ChannelShellAccent = ChannelTitleBarAccent;

export const ChannelShellLayout = memo(function ChannelShellLayout({
  accent,

  children
}: {
  accent: ChannelShellAccent;

  children: ReactNode;
}) {
  const titleBarHeight = useAppSelector((state) => state.app.titleBarHeight);

  return (
    <div className="main-window relative flex min-h-0 flex-1 flex-col">
      <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden">
        <ChannelTitleBar accent={accent} height={titleBarHeight} />

        <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden bg-white dark:bg-[#111]">
          {children}
        </div>
      </div>
    </div>
  );
});
