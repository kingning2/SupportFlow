"use client";

import { useState } from "react";
import { Avatar, Button, Card, Select, Space, Typography } from "@douyinfe/semi-ui-19";
import { IconPlus } from "@douyinfe/semi-icons";

import {
  channelAction,
  localizeChannelText,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import { draftsFromChannel, type ChannelFieldDrafts } from "./channel-fields";
import { WeworkConnectPanel } from "./wework-connect-panel";

const { Title } = Typography;

interface ChannelAddPanelProps {
  catalog: ChannelCatalogEntry[];
  lang: string;
  /** When set (dev preset), skip channel type dropdown. */
  fixedChannel?: string;
  onClose: () => void;
  onConnected: () => void;
}

function emptyDrafts(): ChannelFieldDrafts {
  return { strings: {}, bools: {}, maskedCleared: {} };
}

function resolveDrafts(catalog: ChannelCatalogEntry[], channelName?: string): ChannelFieldDrafts {
  if (!channelName) {
    return emptyDrafts();
  }
  const row = catalog.find((channel) => channel.name === channelName);
  return row ? draftsFromChannel(row) : emptyDrafts();
}

export function ChannelAddPanel({
  catalog,
  lang,
  fixedChannel,
  onClose,
  onConnected
}: ChannelAddPanelProps) {
  const [selected, setSelected] = useState(fixedChannel ?? "");
  const [connecting, setConnecting] = useState(false);
  const [, setDrafts] = useState<ChannelFieldDrafts>(() => resolveDrafts(catalog, fixedChannel));

  const activeNames = new Set(catalog.filter((c) => c.active).map((c) => c.name));
  const available = catalog.filter((c) => !activeNames.has(c.name));
  const selectedChannel = fixedChannel ?? selected;
  const ch = catalog.find((c) => c.name === selectedChannel);

  const pickChannel = (name: string) => {
    setSelected(name);
    setDrafts(resolveDrafts(catalog, name));
  };

  if (available.length === 0) {
    return (
      <Card style={{ marginTop: 16, textAlign: "center" }}>
        <Typography.Text type="tertiary">所有可用通道均已接入</Typography.Text>
        <div style={{ marginTop: 12 }}>
          <Button theme="borderless" type="tertiary" size="small" onClick={onClose}>
            取消
          </Button>
        </div>
      </Card>
    );
  }

  const showWeworkPanel = selectedChannel === "wework" && ch;

  return (
    <Card
      style={{ marginTop: 16, borderColor: "var(--semi-color-primary-light-active)" }}
      bodyStyle={{ padding: 24 }}
    >
      <Space spacing="medium" style={{ marginBottom: 20 }}>
        <Avatar
          size="small"
          style={{
            background: "var(--semi-color-primary-light-default)",
            color: "var(--semi-color-primary)"
          }}
        >
          <IconPlus />
        </Avatar>
        <Title heading={6} style={{ margin: 0 }}>
          接入通道
        </Title>
      </Space>

      {fixedChannel ? null : (
        <div style={{ marginBottom: 16 }}>
          <Select
            placeholder="选择要接入的通道…"
            value={selected || undefined}
            style={{ width: "100%" }}
            onChange={(value) => pickChannel(String(value ?? ""))}
          >
            {available.map((item) => (
              <Select.Option key={item.name} value={item.name}>
                {localizeChannelText(item.label, lang)} ({item.name})
              </Select.Option>
            ))}
          </Select>
        </div>
      )}

      <Space vertical spacing="medium" style={{ width: "100%" }}>
        {showWeworkPanel ? (
          <WeworkConnectPanel
            channel={ch}
            lang={lang}
            connecting={connecting}
            onConnect={async (config) => {
              setConnecting(true);
              try {
                await channelAction({ action: "connect", channel: "wework", config });
                onConnected();
                onClose();
              } catch {
                // keep panel open
              } finally {
                setConnecting(false);
              }
            }}
          />
        ) : null}
      </Space>
    </Card>
  );
}
