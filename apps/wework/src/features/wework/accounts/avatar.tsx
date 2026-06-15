"use client";

import { Avatar } from "@douyinfe/semi-ui-19";

/** 从名称取 1–2 个展示用缩写（支持中英文） */
export function accountInitials(name: string): string {
  const trimmed = name.trim();
  if (!trimmed) {
    return "?";
  }
  const parts = trimmed.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) {
    return (parts[0]![0]! + parts[1]![0]!).toUpperCase();
  }
  if (/[\u4e00-\u9fff]/.test(trimmed)) {
    return trimmed.slice(0, 2);
  }
  return trimmed.slice(0, 2).toUpperCase();
}

/** 名称哈希 → 稳定背景色（HSL） */
export function accountAvatarStyle(name: string): { background: string; color: string } {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = Math.abs(hash) % 360;
  return {
    background: `hsl(${hue} 62% 42%)`,
    color: "#ffffff"
  };
}

export interface AccountAvatarProps {
  name: string;
  size?: "sm" | "md" | "lg";
  className?: string;
}

const SIZE_MAP = {
  sm: "small",
  md: "medium",
  lg: "large"
} as const;

export function AccountAvatar({ name, size = "md", className }: AccountAvatarProps) {
  const initials = accountInitials(name);
  const style = accountAvatarStyle(name);

  return (
    <Avatar className={className} size={SIZE_MAP[size]} style={style}>
      {initials}
    </Avatar>
  );
}
