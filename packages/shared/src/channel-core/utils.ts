import type { ChannelLocalized } from "../channel/types";

/** 多语言已移除，直接取中文或第一个可用值。 */
export function localizeChannelText(value: ChannelLocalized | undefined, _lang?: string): string {
  if (!value) return "";
  if (typeof value === "string") return value;
  return value.zh ?? value.en ?? Object.values(value)[0] ?? "";
}

export function channelFieldValueString(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "boolean") return value ? "true" : "false";
  return String(value);
}

export function isChannelMaskedSecret(value: string) {
  return value.includes("****");
}

export function channelLangFromI18n(language: string): "zh" | "en" {
  return language.startsWith("zh") ? "zh" : "en";
}
