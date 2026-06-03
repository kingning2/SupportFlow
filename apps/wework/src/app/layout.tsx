import type { Metadata } from "next";

import "antd/dist/reset.css";

import { DesktopAppLayout } from "@supportflow/ui/app-shell";
import "@supportflow/ui/design-system";
import "@supportflow/ui/design-system/flavors/wework";

import "@/console/styles/wework-console.css";

import { weworkShellAccent } from "@/shell-accent";
import { WEWORK_SHELL_CONTENT_CLASS } from "@/wework-shell";

export const metadata: Metadata = {
  title: "SupportFlow · 企微个人号",
  description: "企业微信个人号通道"
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="zh-Hans" className="light">
      <body data-flavor="wework" className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <DesktopAppLayout accent={weworkShellAccent} contentClassName={WEWORK_SHELL_CONTENT_CLASS}>
          {children}
        </DesktopAppLayout>
      </body>
    </html>
  );
}
