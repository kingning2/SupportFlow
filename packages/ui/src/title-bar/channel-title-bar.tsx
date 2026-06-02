"use client";

import { Minus, X } from "lucide-react";
import { memo } from "react";

import { mainWindow } from "@supportflow/shared/tauri-bridge/window/main-window";
import { cn } from "@supportflow/shared";

export type ChannelTitleBarAccent = {
  logoGradient: string;
  title: string;
  barClassName: string;
  logoText: string;
};

function handleBarMouseDown(e: React.MouseEvent) {
  const isDragRegion = Boolean((e.target as HTMLElement).dataset.dragRegion);
  if (isDragRegion && e.buttons === 1) {
    void mainWindow.startDragging();
  }
}

export const ChannelTitleBar = memo(function ChannelTitleBar({
  accent,
  height
}: {
  accent: ChannelTitleBarAccent;
  height: number;
}) {
  return (
    <div
      role="banner"
      data-drag-region
      className={cn(
        "flex w-full items-center justify-between px-3 select-none",
        accent.barClassName
      )}
      style={{ height }}
      onMouseDown={handleBarMouseDown}
    >
      <div className="pointer-events-none flex min-w-0 flex-1 items-center gap-2">
        <div
          className={cn(
            "flex size-8 shrink-0 items-center justify-center rounded-lg bg-linear-to-br text-sm font-bold text-white",
            accent.logoGradient
          )}
          aria-hidden
        >
          {accent.logoText}
        </div>
        <span className="truncate text-[15px] font-semibold tracking-tight text-slate-800 dark:text-slate-100">
          {accent.title}
        </span>
      </div>
      <div className="pointer-events-auto flex shrink-0 items-center gap-1">
        <button
          type="button"
          className="inline-flex size-8 cursor-pointer items-center justify-center rounded-md text-slate-500 hover:bg-black/5 dark:hover:bg-white/10"
          aria-label="minimize"
          onClick={() => void mainWindow.minimize()}
        >
          <Minus className="size-4" />
        </button>
        <button
          type="button"
          className="inline-flex size-8 cursor-pointer items-center justify-center rounded-md text-slate-500 hover:bg-red-500/10 hover:text-red-600"
          aria-label="close"
          onClick={() => void mainWindow.close()}
        >
          <X className="size-4" />
        </button>
      </div>
    </div>
  );
});
