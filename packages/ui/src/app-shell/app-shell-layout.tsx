"use client";

import { memo, type ReactNode } from "react";

import { ModalOverlay, ModalWindowProvider } from "@supportflow/ui/modal";
import { TitleBar, type TitleBarAccent } from "@supportflow/ui/title-bar";

import { useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";

import { MainWindowBgGuard } from "./main-window-bg-guard";
import { APP_SHELL_CONTENT_CLASS } from "./shell-content-class";

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
