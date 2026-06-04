/** SupportFlow Agent channel ids (must match Python `ChannelsHandler.CHANNEL_DEFS`). */
export const CHANNEL_IDS = ["wx", "wework"] as const;

export type ChannelCatalogEntryId = (typeof CHANNEL_IDS)[number];

const CHANNEL_ID_SET = new Set<string>(CHANNEL_IDS);

/** CLI / npm script aliases → canonical channel id. */
export const DEV_CHANNEL_ALIASES: Record<string, ChannelCatalogEntryId> = {
  wechat: "wx",
  personal_wechat: "wx",
  wx: "wx",
  wework: "wework"
};

export function isChannelCatalogEntryId(value: string): value is ChannelCatalogEntryId {
  return CHANNEL_ID_SET.has(value);
}

export function resolveDevChannel(raw: string | undefined | null): ChannelCatalogEntryId | null {
  const trimmed = raw?.trim();
  if (!trimmed) {
    return null;
  }
  const lower = trimmed.toLowerCase();
  const aliased = DEV_CHANNEL_ALIASES[lower];
  if (aliased) {
    return aliased;
  }
  if (isChannelCatalogEntryId(lower)) {
    return lower;
  }
  return null;
}

export function channelLabelKey(channelId: ChannelCatalogEntryId): string {
  return `channel_label_${channelId}`;
}
