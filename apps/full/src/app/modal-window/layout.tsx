"use client";

import { getCurrentWindow } from "@tauri-apps/api/window";

import { ModalMotionProvider } from "@supportflow/ui/modal";
import { useIsomorphicLayoutEffect } from "@supportflow/shared/desktop-shell/useIsomorphicLayoutEffect";
import { isModalWindowLabel } from "@/config/windows";

export default function ModalWindowLayout({ children }: { children: React.ReactNode }) {
  useIsomorphicLayoutEffect(() => {
    const label = getCurrentWindow().label;
    if (!isModalWindowLabel(label)) return;
    document.getElementById("App")?.classList.add("modal-window-root");
  }, []);

  return <ModalMotionProvider>{children}</ModalMotionProvider>;
}
