"use client";

import StoreProvider from "@supportflow/shared/desktop-shell/providers/store";

import DesktopRoot from "@supportflow/shared/desktop-shell/providers/desktop-root";

import TauriEventProvider from "@supportflow/shared/desktop-shell/providers/tauri-event-provider";

export function ChannelAppRoot({ children }: { children: React.ReactNode }) {
  return (
    <StoreProvider>
      <TauriEventProvider>
        <DesktopRoot>{children}</DesktopRoot>
      </TauriEventProvider>
    </StoreProvider>
  );
}
