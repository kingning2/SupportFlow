import type { Metadata } from "next";

import { DesktopAppLayout } from "@supportflow/ui/app-shell";
import "@supportflow/ui/design-system";
import "@supportflow/ui/design-system/flavors/wechat";

import { wechatShellAccent } from "@/shell-accent";

export const metadata: Metadata = {
  title: "SupportFlow · 微信个人号",
  description: "微信个人号通道"
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="zh-Hans" className="light">
      <body data-flavor="wechat" className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <DesktopAppLayout accent={wechatShellAccent}>{children}</DesktopAppLayout>
      </body>
    </html>
  );
}
