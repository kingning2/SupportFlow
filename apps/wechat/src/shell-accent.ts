import type { TitleBarAccent } from "@supportflow/ui/title-bar";

/** 微信个人号标题栏品牌（传入统一 `TitleBar`） */
export const wechatShellAccent: TitleBarAccent = {
  logoGradient: "from-[#07C160] to-[#06AD56]",
  title: "SupportFlow · 微信",
  barClassName:
    "border-b border-[#07C160]/20 bg-[#07C160]/5 dark:border-[#07C160]/30 dark:bg-[#07C160]/10",
  logoText: "微"
};
