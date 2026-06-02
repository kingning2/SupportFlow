"use client";

import { ModalOverlay, ModalWindowProvider } from "@supportflow/ui/modal";
import { TitleBar } from "@supportflow/ui/title-bar";
import GlobalBgGuard from "@/guards/main-window/global-bg-guard";
import { useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";

export default function MainProvider({ children }: { children: React.ReactNode }) {
  const titleBarHeight = useAppSelector((state) => state.app.titleBarHeight);

  return (
    <ModalWindowProvider>
      <div className="main-window relative flex min-h-0 flex-1 flex-col">
        <GlobalBgGuard />

        <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden">
          <TitleBar height={titleBarHeight} />
          <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden bg-white p-3">
            {children}
          </div>
          <ModalOverlay />
        </div>
      </div>
    </ModalWindowProvider>
  );
}
