"use client";

import type { ChannelCatalogEntry } from "@supportflow/shared";

export interface PageActions {
  fetchChannels: () => Promise<ChannelCatalogEntry[]>;
  connect: (config: Record<string, string | number | boolean>) => Promise<void>;
  disconnect: () => Promise<void>;
  save: (config: Record<string, string | number | boolean>) => Promise<void>;
  syncContacts: () => Promise<void>;
}
