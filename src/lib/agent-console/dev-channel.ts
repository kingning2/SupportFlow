import { type ChannelCatalogEntryId, resolveDevChannel } from "@/enums/dev-channel";

/**
 * Build-time / dev-server channel lock from `NEXT_PUBLIC_DEV_CHANNEL`.
 * Set via `bun run tauri:dev:wechat` (see `scripts/tauri-dev-channel.mjs`).
 */
export function getDevChannel(): ChannelCatalogEntryId | null {
  return resolveDevChannel(process.env.NEXT_PUBLIC_DEV_CHANNEL);
}
