import type { Metadata } from "next";

import { ChannelAppRoot, ChannelShellLayout } from "@supportflow/ui/app-shell";
import "@supportflow/ui/app-shell/styles.css";

import { weworkShellAccent } from "@/shell-accent";

export const metadata: Metadata = {
  title: "SupportFlow · 企微个人号",
  description: "企业微信个人号通道"
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="zh-Hans">
      <body className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ChannelAppRoot>
          <ChannelShellLayout accent={weworkShellAccent}>{children}</ChannelShellLayout>
        </ChannelAppRoot>
      </body>
    </html>
  );
}
