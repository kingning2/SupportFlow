import type { ChannelLocalized } from "./types";

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
