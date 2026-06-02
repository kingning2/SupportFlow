"use client";

import {
  channelFieldValueString,
  isChannelMaskedSecret,
  localizeChannelText,
  type ChannelCatalogEntry,
  type ChannelField
} from "@/cmd/channel-python-channels";
import { cn } from "@/lib/utils";

export interface ChannelFieldDrafts {
  strings: Record<string, string>;
  bools: Record<string, boolean>;
  maskedCleared: Record<string, boolean>;
}

interface ChannelFieldsProps {
  channelName: string;
  fields: ChannelField[];
  lang: string;
  drafts: ChannelFieldDrafts;
  onChange: (next: ChannelFieldDrafts) => void;
}

export function ChannelFields({ channelName, fields, lang, drafts, onChange }: ChannelFieldsProps) {
  return (
    <div className="space-y-4">
      {fields.map((field) => {
        const fieldLabel = localizeChannelText(field.label, lang);
        const inputId = `ch-${channelName}-${field.key}`;
        const rawVal = channelFieldValueString(field.value);
        const isMasked = isChannelMaskedSecret(rawVal) && !drafts.maskedCleared[field.key];

        if (field.type === "bool" || field.type === "checkbox") {
          const checked = drafts.bools[field.key] ?? rawVal === "true";
          return (
            <div key={field.key}>
              <label
                htmlFor={inputId}
                className="mb-1.5 block text-sm font-medium text-slate-600 dark:text-slate-400"
              >
                {fieldLabel}
              </label>
              <label className="relative inline-flex cursor-pointer items-center">
                <input
                  id={inputId}
                  type="checkbox"
                  className="peer sr-only"
                  checked={checked}
                  onChange={(e) =>
                    onChange({
                      ...drafts,
                      bools: { ...drafts.bools, [field.key]: e.target.checked }
                    })
                  }
                />
                <div
                  className={cn(
                    "h-5 w-9 rounded-full bg-slate-200 after:absolute after:top-[2px] after:left-[2px]",
                    "after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all",
                    "peer-checked:bg-[#4ABE6E] peer-checked:after:translate-x-full dark:bg-slate-700"
                  )}
                />
              </label>
            </div>
          );
        }

        const inputType = field.type === "number" ? "number" : "text";
        const value = isMasked ? rawVal : (drafts.strings[field.key] ?? rawVal);

        return (
          <div key={field.key}>
            <label
              htmlFor={inputId}
              className="mb-1.5 block text-sm font-medium text-slate-600 dark:text-slate-400"
            >
              {fieldLabel}
            </label>
            <input
              id={inputId}
              type={inputType}
              value={value}
              placeholder={fieldLabel}
              className={cn(
                "w-full rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 font-mono text-sm",
                "text-slate-800 transition-colors focus:border-[#35A85B] focus:outline-none",
                "dark:border-slate-600 dark:bg-white/5 dark:text-slate-100",
                isMasked && "cfg-key-masked"
              )}
              onFocus={() => {
                if (isMasked) {
                  onChange({
                    ...drafts,
                    strings: { ...drafts.strings, [field.key]: "" },
                    maskedCleared: { ...drafts.maskedCleared, [field.key]: true }
                  });
                }
              }}
              onChange={(e) =>
                onChange({
                  ...drafts,
                  strings: { ...drafts.strings, [field.key]: e.target.value }
                })
              }
            />
          </div>
        );
      })}
    </div>
  );
}

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
