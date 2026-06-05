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
    iconBox: "bg-channel/10 dark:bg-channel/20",
    icon: "text-channel"
  },
  green: {
    iconBox: "bg-success/10 dark:bg-success/20",
    icon: "text-success"
  },
  blue: {
    iconBox: "bg-info/10 dark:bg-info/20",
    icon: "text-info"
  }
};

export function channelIconComponent(faClass?: string): LucideIcon {
  return CHANNEL_ICON_MAP[faClass ?? ""] ?? MessageCircle;
}

export function channelColorClasses(color?: string) {
  return COLOR_MAP[color ?? "emerald"] ?? COLOR_MAP.emerald;
}
