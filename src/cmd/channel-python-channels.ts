import { invokeWrapper } from "@/cmd/invoke";
import { TauriCmd } from "@/enums/tauri-cmd";

/** Localized string or map (SupportFlow Agent Python API shape). */
export type ChannelLocalized = string | Record<string, string>;

export interface ChannelField {
  key: string;
  label: ChannelLocalized;
  type: string;
  value: unknown;
  default?: unknown;
  placeholder?: ChannelLocalized;
}

export interface ChannelCatalogEntry {
  name: string;
  label: ChannelLocalized;
  active: boolean;
  fields: ChannelField[];
  hint?: ChannelLocalized;
  icon?: string;
  color?: string;
  login_status?: string;
  loginStatus?: string;
}

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

export function channelLangFromI18n(language: string): "zh" | "en" {
  return language.startsWith("zh") ? "zh" : "en";
}

export function localizeChannelText(value: ChannelLocalized | undefined, lang: string): string {
  if (!value) {
    return "";
  }
  if (typeof value === "string") {
    return value;
  }
  return value[lang] ?? value.en ?? value.zh ?? Object.values(value)[0] ?? "";
}

export function channelFieldValueString(value: unknown): string {
  if (value === null || value === undefined) {
    return "";
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return String(value);
}

export function isChannelMaskedSecret(value: string) {
  return value.includes("****");
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

/** Proxy `/api/*` channel console endpoints (QR login, Feishu register). */
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
