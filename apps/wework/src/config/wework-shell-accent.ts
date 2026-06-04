import type { TitleBarAccent } from "@supportflow/ui/title-bar";

/** 企微官网蓝白标题栏（仅浅色，无深色渐变） */
export const weworkShellAccent: TitleBarAccent = {
  logoGradient: "from-[#4A9AFF] to-[#267EF0]",
  title: "SupportFlow · 企微",
  barClassName: "border-b border-[#267EF0]/15 bg-[#F8FBFF]",
  logoText: "企",
  titleClassName: "text-[#1A2B4A]",
  controlClassName: "text-[#267EF0]/85 hover:bg-[#267EF0]/10 hover:text-[#1A5FD9]"
};
