import { invokeWrapper } from "@/cmd/invoke";
import { TauriCmd } from "@/enums/tauri-cmd";

/** Localized string or map (CowAgent Python API shape). */
export type CowLocalized = string | Record<string, string>;

export interface CowChannelField {
  key: string;
  label: CowLocalized;
  type: string;
  value: unknown;
  default?: unknown;
  placeholder?: CowLocalized;
}

export interface CowChannel {
  name: string;
  label: CowLocalized;
  active: boolean;
  fields: CowChannelField[];
  hint?: CowLocalized;
  icon?: string;
  color?: string;
  login_status?: string;
  loginStatus?: string;
}

export type CowConsoleApiMethod = "GET" | "POST";

export interface CowConsoleApiResponse {
  status: string;
  message?: string;
  [key: string]: unknown;
}

export interface CowChannelActionRequest {
  action: "connect" | "disconnect" | "save";
  channel: string;
  config?: Record<string, string | number | boolean>;
}

interface CowChannelsApiResponse {
  status: string;
  channels?: CowChannel[];
  message?: string;
}

interface CowChannelActionApiResponse {
  status: string;
  channel_type?: string;
  restarted?: boolean;
  message?: string;
}

export function cowLangFromI18n(language: string): "zh" | "en" {
  return language.startsWith("zh") ? "zh" : "en";
}

export function localizeCowText(value: CowLocalized | undefined, lang: string): string {
  if (!value) {
    return "";
  }
  if (typeof value === "string") {
    return value;
  }
  return value[lang] ?? value.en ?? value.zh ?? Object.values(value)[0] ?? "";
}

export function cowFieldValueString(value: unknown): string {
  if (value === null || value === undefined) {
    return "";
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return String(value);
}

export function isCowMaskedSecret(value: string) {
  return value.includes("****");
}

export async function fetchCowChannels(): Promise<CowChannel[]> {
  const data = await invokeWrapper<CowChannelsApiResponse>(TauriCmd.AgentGetChannelCatalog);
  if (data.status !== "success" || !Array.isArray(data.channels)) {
    throw new Error(data.message ?? "Failed to load channels");
  }
  return data.channels;
}

export async function cowChannelAction(body: CowChannelActionRequest) {
  const data = await invokeWrapper<CowChannelActionApiResponse>(TauriCmd.AgentChannelAction, {
    body
  });
  if (data.status !== "success") {
    throw new Error(data.message ?? "Channel action failed");
  }
  return data;
}

/** Proxy `/api/*` channel console endpoints (QR login, Feishu register). */
export async function fetchCowChannelConsoleApi(
  path: string,
  method: CowConsoleApiMethod,
  body: Record<string, unknown> = {}
): Promise<CowConsoleApiResponse> {
  return invokeWrapper<CowConsoleApiResponse>(TauriCmd.AgentChannelConsoleApi, {
    body: { path, method, body }
  });
}

export function cowChannelLoginStatus(ch: CowChannel): string | undefined {
  return ch.login_status ?? ch.loginStatus;
}
