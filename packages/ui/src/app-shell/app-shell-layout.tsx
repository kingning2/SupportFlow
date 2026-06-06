"use client";

import { memo, type MouseEvent, type ReactNode } from "react";

import { ModalOverlay, ModalWindowProvider } from "@supportflow/ui/modal";
import { TitleBar, type TitleBarAccent } from "@supportflow/ui/title-bar";

import { cn } from "@supportflow/shared";
import { mainWindow } from "@supportflow/shared/tauri-bridge/window/main-window";
import { useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";

import { MainWindowBgGuard } from "./main-window-bg-guard";
import { APP_SHELL_CONTENT_CLASS } from "./shell-content-class";

type ResizeDirection = Parameters<typeof mainWindow.startResizeDragging>[0];

export type AppShellLayoutProps = {
  children: ReactNode;
  /** 通道品牌色；完整控制台不传 */
  accent?: TitleBarAccent;
  /** 主窗内容区 className，默认 channel */
  contentClassName?: string;
  /** 完整控制台：Modal 蒙层与子窗 */
  modal?: boolean;
  /** 完整控制台：同步 --main-window-bg */
  bgGuard?: boolean;
};

const WINDOW_RESIZE_HIT_AREAS: Array<{
  direction: ResizeDirection;
  className: string;
}> = [
  { direction: "North", className: "top-0 left-3 right-3 h-1 cursor-n-resize" },
  { direction: "South", className: "bottom-0 left-3 right-3 h-1 cursor-s-resize" },
  { direction: "West", className: "top-3 bottom-3 left-0 w-1 cursor-w-resize" },
  { direction: "East", className: "top-3 bottom-3 right-0 w-1 cursor-e-resize" },
  { direction: "NorthWest", className: "top-0 left-0 h-3 w-3 cursor-nw-resize" },
  { direction: "NorthEast", className: "top-0 right-0 h-3 w-3 cursor-ne-resize" },
  { direction: "SouthWest", className: "bottom-0 left-0 h-3 w-3 cursor-sw-resize" },
  { direction: "SouthEast", className: "bottom-0 right-0 h-3 w-3 cursor-se-resize" }
];

/**
 * 触发无边框窗口的系统缩放拖拽。
 *
 * # Arguments
 *
 * * `event` - 当前鼠标按下事件
 * * `direction` - 缩放方向
 */
function handleResizeMouseDown(event: MouseEvent<HTMLDivElement>, direction: ResizeDirection) {
  if (event.buttons !== 1) return;
  event.stopPropagation();
  void mainWindow.startResizeDragging(direction);
}

export const AppShellLayout = memo(function AppShellLayout({
  children,
  accent,
  contentClassName = APP_SHELL_CONTENT_CLASS.channel,
  modal = false,
  bgGuard = false
}: AppShellLayoutProps) {
  const titleBarHeight = useAppSelector((state) => state.app.titleBarHeight);

  const shell = (
    <div className="main-window relative flex min-h-0 flex-1 flex-col">
      {WINDOW_RESIZE_HIT_AREAS.map(({ direction, className }) => (
        <div
          key={direction}
          aria-hidden
          className={cn("absolute z-50 select-none", className)}
          onMouseDown={(event) => handleResizeMouseDown(event, direction)}
        />
      ))}
      {bgGuard ? <MainWindowBgGuard /> : null}
      <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden">
        <TitleBar height={titleBarHeight} accent={accent} />
        <div className={contentClassName}>{children}</div>
        {modal ? <ModalOverlay /> : null}
      </div>
    </div>
  );

  if (modal) {
    return <ModalWindowProvider>{shell}</ModalWindowProvider>;
  }

  return shell;
});
