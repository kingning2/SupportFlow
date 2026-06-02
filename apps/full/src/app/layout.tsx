import type { Metadata } from "next";

import { DesktopAppRoot } from "@supportflow/ui/app-shell";

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
      <body className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <DesktopAppRoot>{children}</DesktopAppRoot>
      </body>
    </html>
  );
}
