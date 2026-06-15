"use client";

import { useMemo, useState } from "react";
import { Button, Checkbox, Input, Space, Typography } from "@douyinfe/semi-ui-19";
import {
  channelFieldValueString,
  localizeChannelText,
  type ChannelCatalogEntry,
  type ChannelFieldDrafts
} from "@supportflow/shared";

const { Text } = Typography;

const WEWORK_DEFAULT_VERSION = "4.0.8.6027";
const WEWORK_DEFAULT_INIT_WAIT_SECONDS = 60;

export interface WeworkConnectPanelProps {
  channel: ChannelCatalogEntry;
  lang: string;
  connecting?: boolean;
  onConnect: (config: Record<string, string | number | boolean>) => void | Promise<void>;
  onCancel?: () => void;
}

function draftsFromWeworkChannel(channel: ChannelCatalogEntry): ChannelFieldDrafts {
  const strings: Record<string, string> = {};
  const bools: Record<string, boolean> = { wework_smart: true };
  for (const field of channel.fields) {
    const raw = channelFieldValueString(field.value);
    if (field.type === "bool" || field.type === "checkbox") {
      bools[field.key] = raw === "true" || raw === "1";
    } else {
      strings[field.key] = raw;
    }
  }
  return { strings, bools, maskedCleared: {} };
}

function buildWeworkConnectConfig(
  drafts: ChannelFieldDrafts
): Record<string, string | number | boolean> {
  const exePath = (drafts.strings.wework_exe_path ?? "").trim();
  return {
    wework_exe_path: exePath,
    wework_smart: drafts.bools.wework_smart ?? true,
    wework_version: WEWORK_DEFAULT_VERSION,
    wework_init_wait_seconds: WEWORK_DEFAULT_INIT_WAIT_SECONDS
  };
}

/** 企微个人号接入表单（仅路径可改，版本/等待时间使用内置默认值） */
export function WeworkConnectPanel({
  channel,
  lang,
  connecting = false,
  onConnect,
  onCancel
}: WeworkConnectPanelProps) {
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() => draftsFromWeworkChannel(channel));

  const pathField = useMemo(
    () => channel.fields.find((f) => f.key === "wework_exe_path"),
    [channel.fields]
  );
  const pathLabel = pathField ? localizeChannelText(pathField.label, lang) : "企微程序路径";
  const pathPlaceholder = pathField?.placeholder
    ? localizeChannelText(pathField.placeholder, lang)
    : pathLabel;
  const pathValue =
    drafts.strings.wework_exe_path ?? (pathField ? channelFieldValueString(pathField.value) : "");

  return (
    <Space vertical align="start" spacing="medium" style={{ width: "100%" }}>
      <Space vertical align="start" spacing="tight" style={{ width: "100%" }}>
        <Text strong>{pathLabel}</Text>
        <Input
          id="wework-exe-path"
          value={pathValue}
          placeholder={pathPlaceholder}
          style={{ width: "100%" }}
          onChange={(value) =>
            setDrafts((prev) => ({
              ...prev,
              strings: { ...prev.strings, wework_exe_path: String(value) }
            }))
          }
        />
      </Space>

      <Checkbox
        checked={drafts.bools.wework_smart ?? true}
        onChange={(e) =>
          setDrafts((prev) => ({
            ...prev,
            bools: { ...prev.bools, wework_smart: Boolean(e.target?.checked) }
          }))
        }
      >
        <Text>复用已登录的企业微信（推荐）</Text>
      </Checkbox>

      <Space style={{ width: "100%", justifyContent: "flex-end" }}>
        {onCancel ? (
          <Button theme="borderless" type="tertiary" disabled={connecting} onClick={onCancel}>
            取消
          </Button>
        ) : null}
        <Button
          type="primary"
          disabled={connecting || !pathValue.trim()}
          loading={connecting}
          onClick={() => void onConnect(buildWeworkConnectConfig(drafts))}
        >
          接入
        </Button>
      </Space>
    </Space>
  );
}
