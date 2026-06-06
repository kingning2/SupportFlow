import type { ChannelCatalogEntry, ChannelFieldDrafts } from "@supportflow/shared";
import { channelFieldValueString, isChannelMaskedSecret } from "@supportflow/shared";

function resolveBooleanFieldValue(channelValue: unknown, draftValue: boolean | undefined): boolean {
  return draftValue ?? channelFieldValueString(channelValue) === "true";
}

function resolveNumberFieldValue(channelValue: unknown, draftValue: string | undefined): number {
  return Number.parseInt(draftValue ?? channelFieldValueString(channelValue), 10) || 0;
}

function shouldSkipSecretField(
  channelValue: unknown,
  fieldKey: string,
  rawDraftValue: string,
  maskedCleared: Record<string, boolean>
): boolean {
  if (isChannelMaskedSecret(rawDraftValue)) {
    return true;
  }

  return isChannelMaskedSecret(channelFieldValueString(channelValue)) && !maskedCleared[fieldKey];
}

function resolveStringFieldValue(channelValue: unknown, draftValue: string): string {
  return draftValue || channelFieldValueString(channelValue);
}

export function buildConfigFromDrafts(
  channel: ChannelCatalogEntry,
  drafts: ChannelFieldDrafts
): Record<string, string | number | boolean> {
  const config: Record<string, string | number | boolean> = {};
  for (const field of channel.fields) {
    if (field.type === "bool" || field.type === "checkbox") {
      config[field.key] = resolveBooleanFieldValue(field.value, drafts.bools[field.key]);
      continue;
    }

    if (field.type === "number") {
      config[field.key] = resolveNumberFieldValue(field.value, drafts.strings[field.key]);
      continue;
    }

    const raw = drafts.strings[field.key] ?? "";
    if (
      field.type === "secret" &&
      shouldSkipSecretField(field.value, field.key, raw, drafts.maskedCleared)
    ) {
      continue;
    }

    config[field.key] = resolveStringFieldValue(field.value, raw);
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
