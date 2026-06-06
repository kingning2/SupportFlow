"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { Loader2, Plus, Radio } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  channelAction,
  channelLangFromI18n,
  fetchChannels,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import { channelLabelKey } from "@supportflow/shared/tauri-bridge/enums";
import { Button } from "@supportflow/ui/button";

import { getDevChannel } from "../lib/agent-console/dev-channel";
import { ActiveChannelCard } from "../views/channels/active-channel-card";
import { ChannelAddPanel } from "../views/channels/channel-add-panel";

function ChannelsEmptyState({ t }: { t: ReturnType<typeof useTranslation>["t"] }) {
  return (
    <div className="flex flex-col items-center justify-center py-20">
      <div className="bg-info/10 mb-4 flex size-16 items-center justify-center rounded-2xl">
        <Radio className="text-info size-7" />
      </div>
      <p className="text-muted-foreground font-medium">{t("channels_empty")}</p>
      <p className="text-muted-foreground mt-1 text-sm">{t("channels_empty_desc")}</p>
    </div>
  );
}

function ChannelsHeader({
  addOpen,
  devChannel,
  loadError,
  onOpenAdd,
  pageDescKey,
  pageTitleKey,
  t
}: {
  addOpen: boolean;
  devChannel: string | null;
  loadError: string | null;
  onOpenAdd: () => void;
  pageDescKey: string;
  pageTitleKey: string;
  t: ReturnType<typeof useTranslation>["t"];
}) {
  return (
    <div className="mb-6 flex items-center justify-between">
      <div>
        <h2 className="text-foreground text-xl font-bold">{t(pageTitleKey)}</h2>
        <p className="text-muted-foreground mt-1 text-sm">{t(pageDescKey)}</p>
      </div>
      {!devChannel && !addOpen ? (
        <Button
          type="button"
          disabled={!!loadError}
          className="flex items-center gap-2"
          onClick={onOpenAdd}
        >
          <Plus className="size-3.5" />
          {t("channels_add")}
        </Button>
      ) : null}
    </div>
  );
}

function ChannelsLoading({ t }: { t: ReturnType<typeof useTranslation>["t"] }) {
  return (
    <div className="text-muted-foreground flex items-center justify-center gap-2 py-8 text-sm">
      <Loader2 className="size-4 animate-spin" />
      <span>{t("channels_loading")}</span>
    </div>
  );
}

function ChannelsError({
  load,
  loadError,
  t
}: {
  load: () => Promise<void>;
  loadError: string;
  t: ReturnType<typeof useTranslation>["t"];
}) {
  return (
    <div className="bg-warning/10 text-warning-foreground border-warning/30 rounded-xl border p-4 text-sm">
      <p className="font-medium">{t("channels_python_unreachable_title")}</p>
      <p className="mt-2 text-xs opacity-90">{loadError}</p>
      <p className="mt-3 text-xs opacity-80">{t("channels_python_unreachable_hint")}</p>
      <Button
        type="button"
        variant="outline"
        className="border-warning/40 mt-4 text-xs"
        onClick={() => void load()}
      >
        {t("channels_retry")}
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
  lang,
  load,
  onCloseAdd,
  showEmptyState,
  t
}: {
  activeChannels: ChannelCatalogEntry[];
  addOpen: boolean;
  catalog: ChannelCatalogEntry[];
  devChannel: string | null;
  handleConnected: () => Promise<void>;
  handleDisconnect: (name: string) => Promise<void>;
  lang: string;
  load: () => Promise<void>;
  onCloseAdd: () => void;
  showEmptyState: boolean;
  t: ReturnType<typeof useTranslation>["t"];
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
            <ChannelsEmptyState t={t} />
          ) : (
            activeChannels.map((channel) => (
              <ActiveChannelCard
                key={channel.name}
                channel={channel}
                lang={lang}
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
          lang={lang}
          fixedChannel={devChannel}
          onClose={onCloseAdd}
          onConnected={() => void handleConnected()}
        />
      ) : null}

      {addOpen && !devChannel ? (
        <ChannelAddPanel
          catalog={catalog}
          lang={lang}
          onClose={onCloseAdd}
          onConnected={() => void handleConnected()}
        />
      ) : null}
    </>
  );
}

export function ChannelsView() {
  const { t, i18n } = useTranslation("console");
  const lang = channelLangFromI18n(i18n.language);
  const devChannel = useMemo(() => getDevChannel(), []);
  const [loading, setLoading] = useState(true);
  const [catalog, setCatalog] = useState<ChannelCatalogEntry[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [addOpen, setAddOpen] = useState(() => devChannel !== null);

  const pageTitleKey = devChannel ? channelLabelKey(devChannel) : "channels_title";
  const pageDescKey = devChannel ? "channels_dev_desc" : "channels_desc";

  const load = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      setCatalog(await fetchChannels());
    } catch (error) {
      setCatalog([]);
      setLoadError(error instanceof Error ? error.message : t("channels_load_failed"));
    } finally {
      setLoading(false);
    }
  }, [t]);

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
      if (!window.confirm(t("channels_disconnect_confirm"))) {
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
    [devChannel, load, t]
  );

  const handleConnected = useCallback(async () => {
    setAddOpen(false);
    await load();
  }, [load]);

  if (devChannel && !loading && !loadError && catalog.length > 0 && !devChannelRow) {
    return (
      <div className="flex h-full min-h-0 flex-col items-center justify-center p-6 text-sm text-red-500">
        <p>{t("channels_dev_unknown", { channel: devChannel })}</p>
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
            pageDescKey={pageDescKey}
            pageTitleKey={pageTitleKey}
            t={t}
          />

          {loading ? <ChannelsLoading t={t} /> : null}
          {!loading && loadError ? <ChannelsError load={load} loadError={loadError} t={t} /> : null}
          {!loading && !loadError ? (
            <ChannelsContent
              activeChannels={activeChannels}
              addOpen={addOpen}
              catalog={catalog}
              devChannel={devChannel}
              handleConnected={handleConnected}
              handleDisconnect={handleDisconnect}
              lang={lang}
              load={load}
              onCloseAdd={() => setAddOpen(false)}
              showEmptyState={showEmptyState}
              t={t}
            />
          ) : null}
        </div>
      </div>
    </div>
  );
}
