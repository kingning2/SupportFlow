"use client";

import { getCurrentWindow } from "@tauri-apps/api/window";

import { useIsomorphicLayoutEffect } from "@supportflow/shared/desktop-shell/useIsomorphicLayoutEffect";
import { ModalMotionProvider } from "@supportflow/ui/modal";

import { isModalWindowLabel } from "@/config/windows";

export function Layout({ children }: { children: React.ReactNode }) {
  useIsomorphicLayoutEffect(() => {
    const label = getCurrentWindow().label;
    if (!isModalWindowLabel(label)) return;
    document.getElementById("App")?.classList.add("modal-window-root");
  }, []);

  return <ModalMotionProvider>{children}</ModalMotionProvider>;
}
