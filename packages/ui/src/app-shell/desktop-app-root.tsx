"use client";

import StoreProvider from "@supportflow/shared/desktop-shell/providers/store";
import DesktopRoot from "@supportflow/shared/desktop-shell/providers/desktop-root";
import TauriEventProvider from "@supportflow/shared/desktop-shell/providers/tauri-event-provider";
import { AntdProvider } from "@supportflow/ui/antd-provider";

/** 桌面应用根：Redux + Tauri 事件 + antd + `#App`（各 flavor 共用） */
export function DesktopAppRoot({ children }: { children: React.ReactNode }) {
  return (
    <StoreProvider>
      <TauriEventProvider>
        <AntdProvider>
          <DesktopRoot>{children}</DesktopRoot>
        </AntdProvider>
      </TauriEventProvider>
    </StoreProvider>
  );
}
