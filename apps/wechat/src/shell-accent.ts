import type { ChannelShellAccent } from "@supportflow/ui/app-shell";

/** 微信个人号专用标题栏样式（仅 wechat 应用引用） */
export const wechatShellAccent: ChannelShellAccent = {
  logoGradient: "from-[#07C160] to-[#06AD56]",
  title: "SupportFlow · 微信",
  barClassName:
    "border-b border-[#07C160]/20 bg-[#07C160]/5 dark:border-[#07C160]/30 dark:bg-[#07C160]/10",
  logoText: "微"
};
