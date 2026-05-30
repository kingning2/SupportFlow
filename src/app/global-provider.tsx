"use client";

import { initWindowConfig } from "@/config/windows";
import { useIsomorphicLayoutEffect } from "@/hooks/useIsomorphicLayoutEffect";

import InitGuard from "@/guards/global/init-guard";
import LanguageGuard from "@/guards/global/language-guard";

export default function GlobalProvider({ children }: { children: React.ReactNode }) {
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
