import { invokeWrapper } from "./invoke";
import { TauriCmd } from "../enums/tauri-cmd";

export type {
  ChannelCatalogEntry,
  ChannelField,
  ChannelFieldDrafts,
  ChannelLocalized
} from "@supportflow/shared";

import type { ChannelCatalogEntry } from "@supportflow/shared";

export {
  channelFieldValueString,
  channelLangFromI18n,
  isChannelMaskedSecret,
  localizeChannelText
} from "@supportflow/shared";

export type ChannelConsoleApiMethod = "GET" | "POST";

export interface ChannelConsoleApiResponse {
  status: string;
  message?: string;
  [key: string]: unknown;
}

export interface ChannelActionRequest {
  action: "connect" | "disconnect" | "save";
  channel: string;
  config?: Record<string, string | number | boolean>;
}

interface ChannelsApiResponse {
  status: string;
  channels?: ChannelCatalogEntry[];
  message?: string;
}

interface ChannelActionApiResponse {
  status: string;
  channel_type?: string;
  restarted?: boolean;
  message?: string;
}

export async function fetchChannels(): Promise<ChannelCatalogEntry[]> {
  const data = await invokeWrapper<ChannelsApiResponse>(TauriCmd.AgentGetChannelCatalog);
  if (data.status !== "success" || !Array.isArray(data.channels)) {
    throw new Error(data.message ?? "Failed to load channels");
  }
  return data.channels;
}

export async function channelAction(body: ChannelActionRequest) {
  const data = await invokeWrapper<ChannelActionApiResponse>(TauriCmd.AgentChannelAction, {
    body
  });
  if (data.status !== "success") {
    throw new Error(data.message ?? "Channel action failed");
  }
  return data;
}

/** Proxy `/api/*` channel console endpoints (WX QR login, WeWork contact sync). */
export async function fetchChannelConsoleApi(
  path: string,
  method: ChannelConsoleApiMethod,
  body: Record<string, unknown> = {}
): Promise<ChannelConsoleApiResponse> {
  return invokeWrapper<ChannelConsoleApiResponse>(TauriCmd.AgentChannelConsoleApi, {
    body: { path, method, body }
  });
}

export function channelLoginStatus(ch: ChannelCatalogEntry): string | undefined {
  return ch.login_status ?? ch.loginStatus;
}
