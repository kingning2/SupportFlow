import type { Metadata } from "next";

import { DesktopAppLayout } from "@supportflow/ui/app-shell";
import "@supportflow/ui/design-system";

import { __CHANNEL_CAMEL__ShellAccent } from "@/config/shell-accent";
import { __CHANNEL_CONST___SHELL_CONTENT_CLASS } from "@/config/shell";
import "@/assets/globals.css";

export const metadata: Metadata = {
  title: "__CHANNEL_TITLE__",
  description: "__CHANNEL_DESCRIPTION__"
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="zh-Hans" className="light">
      <body data-flavor="__CHANNEL_SLUG__" className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <DesktopAppLayout
          accent={__CHANNEL_CAMEL__ShellAccent}
          contentClassName={__CHANNEL_CONST___SHELL_CONTENT_CLASS}
        >
          {children}
        </DesktopAppLayout>
      </body>
    </html>
  );
}
