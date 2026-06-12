"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { Loader2, Plus, Radio } from "lucide-react";

import {
  channelAction,
  fetchChannels,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import { channelLabel } from "@supportflow/shared/tauri-bridge/enums";
import { Button } from "@supportflow/ui/button";

import { getDevChannel } from "../lib/agent-console/dev-channel";
import { ActiveChannelCard } from "../views/channels/active-channel-card";
import { ChannelAddPanel } from "../views/channels/channel-add-panel";

function ChannelsEmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-20">
      <div className="bg-info/10 mb-4 flex size-16 items-center justify-center rounded-2xl">
        <Radio className="text-info size-7" />
      </div>
      <p className="text-muted-foreground font-medium">暂未接入任何通道</p>
      <p className="text-muted-foreground mt-1 text-sm">点击右上角“接入通道”开始配置。</p>
    </div>
  );
}

function ChannelsHeader({
  addOpen,
  devChannel,
  loadError,
  onOpenAdd,
  pageDescription,
  pageTitle
}: {
  addOpen: boolean;
  devChannel: string | null;
  loadError: string | null;
  onOpenAdd: () => void;
  pageDescription: string;
  pageTitle: string;
}) {
  return (
    <div className="mb-6 flex items-center justify-between">
      <div>
        <h2 className="text-foreground text-xl font-bold">{pageTitle}</h2>
        <p className="text-muted-foreground mt-1 text-sm">{pageDescription}</p>
      </div>
      {!devChannel && !addOpen ? (
        <Button
          type="button"
          disabled={!!loadError}
          className="flex items-center gap-2"
          onClick={onOpenAdd}
        >
          <Plus className="size-3.5" />
          接入通道
        </Button>
      ) : null}
    </div>
  );
}

function ChannelsLoading() {
  return (
    <div className="text-muted-foreground flex items-center justify-center gap-2 py-8 text-sm">
      <Loader2 className="size-4 animate-spin" />
      <span>加载通道配置中...</span>
    </div>
  );
}

function ChannelsError({ load, loadError }: { load: () => Promise<void>; loadError: string }) {
  return (
    <div className="bg-warning/10 text-warning-foreground border-warning/30 rounded-xl border p-4 text-sm">
      <p className="font-medium">通道 sidecar 未就绪</p>
      <p className="mt-2 text-xs opacity-90">{loadError}</p>
      <p className="mt-3 text-xs opacity-80">
        请先运行 `pnpm run build:channel-sidecar` 生成 sidecar，开发态也可直接使用 `channel_agent`
        源码。
      </p>
      <Button
        type="button"
        variant="outline"
        className="border-warning/40 mt-4 text-xs"
        onClick={() => void load()}
      >
        重试
      </Button>
    </div>
  );
}

function ChannelsContent({
  activeChannels,
  addOpen,
  catalog,
  devChannel,
  handleConnected,
  handleDisconnect,
  load,
  onCloseAdd,
  showEmptyState
}: {
  activeChannels: ChannelCatalogEntry[];
  addOpen: boolean;
  catalog: ChannelCatalogEntry[];
  devChannel: string | null;
  handleConnected: () => Promise<void>;
  handleDisconnect: (name: string) => Promise<void>;
  load: () => Promise<void>;
  onCloseAdd: () => void;
  showEmptyState: boolean;
}) {
  const showGrid = !((addOpen || devChannel) && activeChannels.length === 0);
  const showFixedAddPanel = Boolean(
    (addOpen || (devChannel && activeChannels.length === 0)) && devChannel
  );

  return (
    <>
      {showGrid ? (
        <div className="grid gap-4">
          {showEmptyState ? (
            <ChannelsEmptyState />
          ) : (
            activeChannels.map((channel) => (
              <ActiveChannelCard
                key={channel.name}
                channel={channel}
                lang="zh"
                onRefresh={() => void load()}
                onDisconnect={(name) => void handleDisconnect(name)}
              />
            ))
          )}
        </div>
      ) : null}

      {showFixedAddPanel && devChannel ? (
        <ChannelAddPanel
          key={devChannel}
          catalog={catalog}
          lang="zh"
          fixedChannel={devChannel}
          onClose={onCloseAdd}
          onConnected={() => void handleConnected()}
        />
      ) : null}

      {addOpen && !devChannel ? (
        <ChannelAddPanel
          catalog={catalog}
          lang="zh"
          onClose={onCloseAdd}
          onConnected={() => void handleConnected()}
        />
      ) : null}
    </>
  );
}

export function Channels() {
  const devChannel = useMemo(() => getDevChannel(), []);
  const [loading, setLoading] = useState(true);
  const [catalog, setCatalog] = useState<ChannelCatalogEntry[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [addOpen, setAddOpen] = useState(() => devChannel !== null);

  const pageTitle = devChannel ? channelLabel(devChannel) : "通道";
  const pageDescription = devChannel
    ? "当前只展示开发指定通道。"
    : "管理本地已接入的通道连接与配置。";

  const load = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      setCatalog(await fetchChannels());
    } catch (error) {
      setCatalog([]);
      setLoadError(error instanceof Error ? error.message : "加载通道列表失败");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    queueMicrotask(() => {
      void load();
    });
  }, [load]);

  const scopedCatalog = useMemo(
    () => (devChannel ? catalog.filter((channel) => channel.name === devChannel) : catalog),
    [catalog, devChannel]
  );
  const activeChannels = scopedCatalog.filter((channel) => channel.active);
  const devChannelRow = devChannel
    ? catalog.find((channel) => channel.name === devChannel)
    : undefined;
  const showEmptyState = activeChannels.length === 0 && !addOpen && !devChannel;

  const handleDisconnect = useCallback(
    async (name: string) => {
      if (!window.confirm("确认断开该通道吗？配置会保留，但通道将停止运行。")) {
        return;
      }
      try {
        await channelAction({ action: "disconnect", channel: name });
        if (devChannel) {
          setAddOpen(true);
        }
        await load();
      } catch {
        // noop
      }
    },
    [devChannel, load]
  );

  const handleConnected = useCallback(async () => {
    setAddOpen(false);
    await load();
  }, [load]);

  if (devChannel && !loading && !loadError && catalog.length > 0 && !devChannelRow) {
    return (
      <div className="flex h-full min-h-0 flex-col items-center justify-center p-6 text-sm text-red-500">
        <p>{`未知通道“${devChannel}”，请检查 NEXT_PUBLIC_DEV_CHANNEL 或启动脚本。`}</p>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-4xl">
          <ChannelsHeader
            addOpen={addOpen}
            devChannel={devChannel}
            loadError={loadError}
            onOpenAdd={() => setAddOpen(true)}
            pageDescription={pageDescription}
            pageTitle={pageTitle}
          />

          {loading ? <ChannelsLoading /> : null}
          {!loading && loadError ? <ChannelsError load={load} loadError={loadError} /> : null}
          {!loading && !loadError ? (
            <ChannelsContent
              activeChannels={activeChannels}
              addOpen={addOpen}
              catalog={catalog}
              devChannel={devChannel}
              handleConnected={handleConnected}
              handleDisconnect={handleDisconnect}
              load={load}
              onCloseAdd={() => setAddOpen(false)}
              showEmptyState={showEmptyState}
            />
          ) : null}
        </div>
      </div>
    </div>
  );
}
