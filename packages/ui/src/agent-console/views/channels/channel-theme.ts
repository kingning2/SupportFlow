import type { LucideIcon } from "lucide-react";
import {
  Bot,
  Building2,
  MessageCircle,
  MessageSquare,
  MessagesSquare,
  Monitor,
  Radio,
  Send
} from "lucide-react";

export const CHANNEL_ICON_MAP: Record<string, LucideIcon> = {
  "fa-tower-broadcast": Radio,
  "fa-comment": MessageCircle,
  "fa-brands fa-weixin": MessageCircle,
  "fa-paper-plane": Send,
  "fa-comments": MessagesSquare,
  "fa-robot": Bot,
  "fa-desktop": Monitor,
  "fa-building": Building2,
  "fa-comment-dots": MessageSquare
};

const COLOR_MAP: Record<string, { iconBox: string; icon: string }> = {
  emerald: {
    iconBox: "bg-emerald-50 dark:bg-emerald-900/20",
    icon: "text-emerald-500"
  },
  green: {
    iconBox: "bg-green-50 dark:bg-green-900/20",
    icon: "text-green-500"
  },
  blue: {
    iconBox: "bg-blue-50 dark:bg-blue-900/20",
    icon: "text-blue-500"
  }
};

export function channelIconComponent(faClass?: string): LucideIcon {
  return CHANNEL_ICON_MAP[faClass ?? ""] ?? MessageCircle;
}

export function channelColorClasses(color?: string) {
  return COLOR_MAP[color ?? "emerald"] ?? COLOR_MAP.emerald;
}
