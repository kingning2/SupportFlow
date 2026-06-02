import type { Metadata } from "next";

import StoreProvider from "@supportflow/shared/desktop-shell/providers/store";
import TauriEventProvider from "@supportflow/shared/desktop-shell/providers/tauri-event-provider";

import GlobalProvider from "./global-provider";

import "@/assets/globals.css";

export const metadata: Metadata = {
  title: "MobiXpert",
  description: "iOS utility toolkit"
};

export default function RootLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-Hant">
      <body>
        <StoreProvider>
          <TauriEventProvider>
            <GlobalProvider>{children}</GlobalProvider>
          </TauriEventProvider>
        </StoreProvider>
      </body>
    </html>
  );
}
