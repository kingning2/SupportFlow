"use client";

import StoreProvider from "@supportflow/shared/desktop-shell/providers/store";
import DesktopRoot from "@supportflow/shared/desktop-shell/providers/desktop-root";
import TauriEventProvider from "@supportflow/shared/desktop-shell/providers/tauri-event-provider";
import { SemiProvider } from "@supportflow/ui/semi-provider";
import { LicenseGateProvider } from "@supportflow/ui/license";

/** 桌面应用根：Redux + Tauri 事件 + Semi Design（飞书主题）+ `#App` */
export function DesktopAppRoot({ children }: { children: React.ReactNode }) {
  return (
    <StoreProvider>
      <TauriEventProvider>
        <SemiProvider>
          <LicenseGateProvider>
            <DesktopRoot>{children}</DesktopRoot>
          </LicenseGateProvider>
        </SemiProvider>
      </TauriEventProvider>
    </StoreProvider>
  );
}
