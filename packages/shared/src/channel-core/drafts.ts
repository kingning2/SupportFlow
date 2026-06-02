import type { ChannelCatalogEntry, ChannelFieldDrafts } from "@supportflow/shared";
import { channelFieldValueString, isChannelMaskedSecret } from "@supportflow/shared";

export function buildConfigFromDrafts(
  channel: ChannelCatalogEntry,
  drafts: ChannelFieldDrafts
): Record<string, string | number | boolean> {
  const config: Record<string, string | number | boolean> = {};
  for (const field of channel.fields) {
    if (field.type === "bool" || field.type === "checkbox") {
      config[field.key] =
        drafts.bools[field.key] ?? channelFieldValueString(field.value) === "true";
    } else if (field.type === "number") {
      config[field.key] =
        Number.parseInt(drafts.strings[field.key] ?? channelFieldValueString(field.value), 10) || 0;
    } else {
      const raw = drafts.strings[field.key] ?? "";
      if (field.type === "secret" && isChannelMaskedSecret(raw)) {
        continue;
      }
      if (
        field.type === "secret" &&
        isChannelMaskedSecret(channelFieldValueString(field.value)) &&
        !drafts.maskedCleared[field.key]
      ) {
        continue;
      }
      config[field.key] = raw || channelFieldValueString(field.value);
    }
  }
  return config;
}

export function draftsFromChannel(channel: ChannelCatalogEntry): ChannelFieldDrafts {
  const strings: Record<string, string> = {};
  const bools: Record<string, boolean> = {};
  const maskedCleared: Record<string, boolean> = {};
  for (const field of channel.fields) {
    const raw = channelFieldValueString(field.value);
    if (field.type === "bool" || field.type === "checkbox") {
      bools[field.key] = raw === "true";
    } else if (!isChannelMaskedSecret(raw)) {
      strings[field.key] = raw;
    } else {
      strings[field.key] = "";
    }
  }
  return { strings, bools, maskedCleared };
}
