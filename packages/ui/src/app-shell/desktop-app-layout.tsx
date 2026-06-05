"use client";

import type { ReactNode } from "react";

import type { TitleBarAccent } from "@supportflow/ui/title-bar";

import { AppShellLayout } from "./app-shell-layout";
import { DesktopAppRoot } from "./desktop-app-root";
import { APP_SHELL_CONTENT_CLASS } from "./shell-content-class";

export type DesktopAppLayoutProps = {
  children: ReactNode;
  accent?: TitleBarAccent;
  contentClassName?: string;
};

/**
 * 单通道 flavor 根布局：`DesktopAppRoot` + `AppShellLayout`（标题栏 + 圆角壳）。
 * 完整控制台在 `main-window/layout` 使用 `AppShellLayout` + `modal` / `bgGuard`。
 */
export function DesktopAppLayout({
  children,
  accent,
  contentClassName = APP_SHELL_CONTENT_CLASS.channel
}: DesktopAppLayoutProps) {
  return (
    <DesktopAppRoot>
      <AppShellLayout accent={accent} contentClassName={contentClassName}>
        {children}
      </AppShellLayout>
    </DesktopAppRoot>
  );
}
