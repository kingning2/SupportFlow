"use client";

import { useState } from "react";
import { Avatar, Button, Card, Space, Tag, Typography } from "@douyinfe/semi-ui-19";

import {
  channelAction,
  localizeChannelText,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import { ChannelHint } from "./channel-hint";
import {
  buildConfigFromDrafts,
  ChannelFields,
  draftsFromChannel,
  type ChannelFieldDrafts
} from "./channel-fields";
import { channelColorStyle, channelIconNode } from "./channel-theme";

const { Text } = Typography;

interface ActiveChannelCardProps {
  channel: ChannelCatalogEntry;
  lang: string;
  onRefresh: () => void;
  onDisconnect: (name: string) => void;
}

function ChannelStatusBadge() {
  return (
    <Tag color="green" size="small" type="light">
      已接入
    </Tag>
  );
}

export function ActiveChannelCard({
  channel,
  lang,
  onRefresh,
  onDisconnect
}: ActiveChannelCardProps) {
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() => draftsFromChannel(channel));
  const [saving, setSaving] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [statusError, setStatusError] = useState(false);

  const colors = channelColorStyle(channel.color);
  const icon = channelIconNode(channel.icon);
  const label = localizeChannelText(channel.label, lang);

  const hasFields = channel.fields.length > 0;
  const showSaveBlock = hasFields;

  const handleSave = async () => {
    setSaving(true);
    try {
      const data = await channelAction({
        action: "save",
        channel: channel.name,
        config: buildConfigFromDrafts(channel, drafts)
      });
      setStatusMsg(data.restarted ? "已保存并重启通道" : "配置已保存");
      setStatusError(false);
      setTimeout(() => setStatusMsg(null), 2500);
      onRefresh();
    } catch {
      setStatusMsg("保存失败");
      setStatusError(true);
      setTimeout(() => setStatusMsg(null), 2500);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div id={`channel-card-${channel.name}`}>
      <Card bodyStyle={{ padding: 24 }}>
        <Space style={{ width: "100%", marginBottom: showSaveBlock ? 20 : 0 }}>
          <Avatar
            size="medium"
            style={{
              background: colors.iconBox,
              color: colors.icon,
              flexShrink: 0
            }}
          >
            {icon}
          </Avatar>
          <div style={{ minWidth: 0, flex: 1 }}>
            <Space spacing="tight">
              <Text strong>{label}</Text>
              <ChannelStatusBadge />
            </Space>
            <Text type="tertiary" size="small" code style={{ display: "block", marginTop: 2 }}>
              {channel.name}
            </Text>
          </div>
          <Button
            type="danger"
            theme="solid"
            size="small"
            onClick={() => onDisconnect(channel.name)}
          >
            断开
          </Button>
        </Space>

        {showSaveBlock ? (
          <Space vertical spacing="medium" style={{ width: "100%" }}>
            {channel.hint ? <ChannelHint hint={channel.hint} lang={lang} /> : null}
            <ChannelFields
              channelName={channel.name}
              fields={channel.fields}
              lang={lang}
              drafts={drafts}
              onChange={setDrafts}
            />
            <Space style={{ width: "100%", justifyContent: "flex-end" }}>
              <Text
                size="small"
                type={statusError ? "danger" : "success"}
                style={{ opacity: statusMsg ? 1 : 0, transition: "opacity 0.3s" }}
              >
                {statusMsg}
              </Text>
              <Button loading={saving} onClick={() => void handleSave()}>
                {saving ? "保存中..." : "保存配置"}
              </Button>
            </Space>
          </Space>
        ) : null}
      </Card>
    </div>
  );
}
