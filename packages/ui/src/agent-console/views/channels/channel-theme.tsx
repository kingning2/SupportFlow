import type { ReactNode } from "react";
import {
  IconApartment,
  IconComment,
  IconCommentStroked,
  IconConnectionPoint2,
  IconDesktop,
  IconSend,
  IconServer,
  IconUserGroup
} from "@douyinfe/semi-icons";

export const CHANNEL_ICON_MAP: Record<string, ReactNode> = {
  "fa-tower-broadcast": <IconConnectionPoint2 />,
  "fa-comment": <IconComment />,
  "fa-brands fa-weixin": <IconCommentStroked />,
  "fa-paper-plane": <IconSend />,
  "fa-comments": <IconUserGroup />,
  "fa-robot": <IconServer />,
  "fa-desktop": <IconDesktop />,
  "fa-building": <IconApartment />,
  "fa-comment-dots": <IconCommentStroked />
};

const COLOR_MAP: Record<string, { iconBox: string; icon: string }> = {
  emerald: {
    iconBox: "var(--semi-color-primary-light-default)",
    icon: "var(--semi-color-primary)"
  },
  green: { iconBox: "var(--semi-color-success-light-default)", icon: "var(--semi-color-success)" },
  blue: { iconBox: "var(--semi-color-info-light-default)", icon: "var(--semi-color-info)" }
};

export function channelIconNode(faClass?: string): ReactNode {
  return CHANNEL_ICON_MAP[faClass ?? ""] ?? <IconComment />;
}

export function channelColorStyle(color?: string) {
  return COLOR_MAP[color ?? "emerald"] ?? COLOR_MAP.emerald;
}
