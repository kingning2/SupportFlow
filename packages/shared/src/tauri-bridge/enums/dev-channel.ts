/** SupportFlow Agent channel ids (must match Python `ChannelsHandler.CHANNEL_DEFS`). */
export const CHANNEL_IDS = ["wework"] as const;

export type ChannelCatalogEntryId = (typeof CHANNEL_IDS)[number];

const CHANNEL_ID_SET = new Set<string>(CHANNEL_IDS);

/** CLI / npm script aliases to canonical channel id. */
export const DEV_CHANNEL_ALIASES: Record<string, ChannelCatalogEntryId> = {
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

export function channelLabel(channelId: ChannelCatalogEntryId): string {
  switch (channelId) {
    case "wework":
      return "企业微信";
    default:
      return channelId;
  }
}
