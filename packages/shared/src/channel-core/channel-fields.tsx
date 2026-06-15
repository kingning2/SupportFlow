"use client";

import { Input, Space, Switch, Typography } from "@douyinfe/semi-ui-19";

import {
  channelFieldValueString,
  isChannelMaskedSecret,
  localizeChannelText,
  type ChannelField,
  type ChannelFieldDrafts
} from "@supportflow/shared";

const { Text } = Typography;

interface ChannelFieldsProps {
  channelName: string;
  fields: ChannelField[];
  lang: string;
  drafts: ChannelFieldDrafts;
  onChange: (next: ChannelFieldDrafts) => void;
}

export function ChannelFields({ channelName, fields, lang, drafts, onChange }: ChannelFieldsProps) {
  return (
    <Space vertical spacing="medium" style={{ width: "100%" }}>
      {fields.map((field) => {
        const fieldLabel = localizeChannelText(field.label, lang);
        const inputId = `ch-${channelName}-${field.key}`;
        const rawVal = channelFieldValueString(field.value);
        const isMasked = isChannelMaskedSecret(rawVal) && !drafts.maskedCleared[field.key];

        if (field.type === "bool" || field.type === "checkbox") {
          const checked = drafts.bools[field.key] ?? rawVal === "true";
          return (
            <Space key={field.key} vertical align="start" spacing="tight" style={{ width: "100%" }}>
              <Text strong id={inputId}>
                {fieldLabel}
              </Text>
              <Switch
                aria-labelledby={inputId}
                checked={checked}
                onChange={(nextChecked) =>
                  onChange({
                    ...drafts,
                    bools: { ...drafts.bools, [field.key]: nextChecked }
                  })
                }
              />
            </Space>
          );
        }

        const inputType = field.type === "number" ? "number" : "text";
        const value = isMasked ? rawVal : (drafts.strings[field.key] ?? rawVal);

        return (
          <Space key={field.key} vertical align="start" spacing="tight" style={{ width: "100%" }}>
            <Text strong id={inputId}>
              {fieldLabel}
            </Text>
            <Input
              id={inputId}
              type={inputType}
              value={value}
              placeholder={fieldLabel}
              style={{
                width: "100%",
                fontFamily: "monospace",
                ...(isMasked ? { WebkitTextSecurity: "disc" } : {})
              }}
              onFocus={() => {
                if (isMasked) {
                  onChange({
                    ...drafts,
                    strings: { ...drafts.strings, [field.key]: "" },
                    maskedCleared: { ...drafts.maskedCleared, [field.key]: true }
                  });
                }
              }}
              onChange={(nextValue) =>
                onChange({
                  ...drafts,
                  strings: { ...drafts.strings, [field.key]: String(nextValue) }
                })
              }
            />
          </Space>
        );
      })}
    </Space>
  );
}
