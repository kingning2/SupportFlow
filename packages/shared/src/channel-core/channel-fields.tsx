"use client";

import {
  channelFieldValueString,
  cn,
  isChannelMaskedSecret,
  localizeChannelText,
  type ChannelField,
  type ChannelFieldDrafts
} from "@supportflow/shared";

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
              <label htmlFor={inputId} className="mb-1.5 block text-sm font-medium text-slate-700">
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
                    "peer-checked:bg-[var(--channel-primary,#35A85B)] peer-checked:after:translate-x-full"
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
            <label htmlFor={inputId} className="mb-1.5 block text-sm font-medium text-slate-700">
              {fieldLabel}
            </label>
            <input
              id={inputId}
              type={inputType}
              value={value}
              placeholder={fieldLabel}
              className={cn(
                "w-full rounded-lg border border-slate-200 bg-white px-3 py-2 font-mono text-sm",
                "text-slate-800 transition-colors focus:border-[var(--channel-primary,#35A85B)] focus:ring-1 focus:ring-[var(--channel-primary,#35A85B)]/25 focus:outline-none",
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
