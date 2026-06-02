"use client";

import { initWindowConfig } from "@supportflow/shared/tauri-bridge/window/main-window";

import InitGuard from "../guards/global/init-guard";
import LanguageGuard from "../guards/global/language-guard";
import { useIsomorphicLayoutEffect } from "../useIsomorphicLayoutEffect";

/** 桌面应用根节点：`#App` 容器 + 窗口初始化 + Init/Language 守卫 */
export default function DesktopRoot({ children }: { children: React.ReactNode }) {
  useIsomorphicLayoutEffect(() => {
    initWindowConfig();

    const app = document.getElementById("App");
    if (!app) return;

    const ua = navigator.userAgent;
    if (/Windows/i.test(ua)) {
      app.classList.add("windows");
    } else if (/Mac/i.test(ua)) {
      app.classList.add("macos");
    }
  }, []);

  return (
    <InitGuard>
      <LanguageGuard>
        <div id="App" className="antialiased" onContextMenu={(e) => e.preventDefault()}>
          {children}
        </div>
      </LanguageGuard>
    </InitGuard>
  );
}
