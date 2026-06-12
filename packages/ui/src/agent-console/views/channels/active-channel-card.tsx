"use client";

import { useState } from "react";
import { MessageCircle } from "lucide-react";

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
import { CHANNEL_ICON_MAP, channelColorClasses } from "./channel-theme";
import { Button } from "@supportflow/ui/button";

interface ActiveChannelCardProps {
  channel: ChannelCatalogEntry;
  lang: string;
  onRefresh: () => void;
  onDisconnect: (name: string) => void;
}

function ChannelStatusBadge() {
  return (
    <>
      <span className="bg-success size-2 rounded-full" />
      <span className="text-success text-xs">已接入</span>
    </>
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

  const colors = channelColorClasses(channel.color);
  const Icon = CHANNEL_ICON_MAP[channel.icon ?? ""] ?? MessageCircle;
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

  const headerMb = hasFields ? "mb-5" : "";

  return (
    <div
      id={`channel-card-${channel.name}`}
      className="bg-card border-border rounded-xl border p-6"
    >
      <div className={`flex items-center gap-4${headerMb ? ` ${headerMb}` : ""}`}>
        <div
          className={`flex size-10 shrink-0 items-center justify-center rounded-xl ${colors.iconBox}`}
        >
          <Icon className={`size-4 ${colors.icon}`} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-foreground font-semibold">{label}</span>
            <ChannelStatusBadge />
          </div>
          <p className="text-muted-foreground mt-0.5 font-mono text-xs">{channel.name}</p>
        </div>
        <Button
          type="button"
          variant="destructive"
          className="h-auto px-3 py-1.5 text-xs"
          onClick={() => onDisconnect(channel.name)}
        >
          断开
        </Button>
      </div>

      {showSaveBlock ? (
        <div className="space-y-4">
          {channel.hint ? <ChannelHint hint={channel.hint} lang={lang} /> : null}
          <ChannelFields
            channelName={channel.name}
            fields={channel.fields}
            lang={lang}
            drafts={drafts}
            onChange={setDrafts}
          />
          <div className="flex items-center justify-end gap-3 pt-1">
            <span
              className={`text-xs transition-opacity duration-300 ${
                statusMsg ? "opacity-100" : "opacity-0"
              } ${statusError ? "text-destructive" : "text-success"}`}
            >
              {statusMsg}
            </span>
            <Button type="button" disabled={saving} onClick={() => void handleSave()}>
              {saving ? "保存中..." : "保存配置"}
            </Button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
