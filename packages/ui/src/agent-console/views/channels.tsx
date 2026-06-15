"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Banner,
  Button,
  Empty,
  Layout,
  Modal,
  Space,
  Spin,
  Typography
} from "@douyinfe/semi-ui-19";
import { IconPlus, IconRadio } from "@douyinfe/semi-icons";

import {
  channelAction,
  fetchChannels,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import { channelLabel } from "@supportflow/shared/tauri-bridge/enums";

import { getDevChannel } from "../lib/agent-console/dev-channel";
import { ActiveChannelCard } from "../views/channels/active-channel-card";
import { ChannelAddPanel } from "../views/channels/channel-add-panel";

const { Title, Text } = Typography;
const { Content } = Layout;

function ChannelsEmptyState() {
  return (
    <Empty
      image={<IconRadio size="extra-large" style={{ color: "var(--semi-color-info)" }} />}
      title="暂未接入任何通道"
      description="点击右上角「接入通道」开始配置。"
      style={{ padding: "48px 0" }}
    />
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
    <div
      style={{
        marginBottom: 24,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between"
      }}
    >
      <div>
        <Title heading={5} style={{ margin: 0 }}>
          {pageTitle}
        </Title>
        <Text type="tertiary" size="small" style={{ display: "block", marginTop: 4 }}>
          {pageDescription}
        </Text>
      </div>
      {!devChannel && !addOpen ? (
        <Button icon={<IconPlus />} disabled={!!loadError} onClick={onOpenAdd}>
          接入通道
        </Button>
      ) : null}
    </div>
  );
}

function ChannelsLoading() {
  return (
    <Space style={{ justifyContent: "center", width: "100%", padding: "32px 0" }}>
      <Spin tip="加载通道配置中..." />
    </Space>
  );
}

function ChannelsError({ load, loadError }: { load: () => Promise<void>; loadError: string }) {
  return (
    <Banner
      type="warning"
      fullMode={false}
      bordered
      closeIcon={null}
      title="通道 sidecar 未就绪"
      description={
        <Space vertical align="start" spacing="tight">
          <Text size="small">{loadError}</Text>
          <Text type="tertiary" size="small">
            请先运行 `pnpm run build:channel-sidecar` 生成 sidecar，开发态也可直接使用 channel_agent
            源码。
          </Text>
          <Button size="small" onClick={() => void load()}>
            重试
          </Button>
        </Space>
      }
    />
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
  handleDisconnect: (name: string) => void;
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
        <Space vertical spacing="medium" style={{ width: "100%" }}>
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
        </Space>
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
    (name: string) => {
      Modal.confirm({
        title: "确认断开该通道吗？",
        content: "配置会保留，但通道将停止运行。",
        okType: "danger",
        onOk: async () => {
          try {
            await channelAction({ action: "disconnect", channel: name });
            if (devChannel) {
              setAddOpen(true);
            }
            await load();
          } catch {
            // noop
          }
        }
      });
    },
    [devChannel, load]
  );

  const handleConnected = useCallback(async () => {
    setAddOpen(false);
    await load();
  }, [load]);

  if (devChannel && !loading && !loadError && catalog.length > 0 && !devChannelRow) {
    return (
      <Layout style={{ height: "100%", minHeight: 0 }}>
        <Content
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            padding: 24,
            color: "var(--semi-color-danger)"
          }}
        >
          <Text>{`未知通道「${devChannel}」，请检查 NEXT_PUBLIC_DEV_CHANNEL 或启动脚本。`}</Text>
        </Content>
      </Layout>
    );
  }

  return (
    <Layout style={{ height: "100%", minHeight: 0, overflow: "hidden" }}>
      <Content style={{ flex: 1, minHeight: 0, overflow: "auto", padding: 24 }}>
        <div style={{ maxWidth: 896, margin: "0 auto" }}>
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
      </Content>
    </Layout>
  );
}
