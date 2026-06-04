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
import { ActiveChannelCard } from "../views/channels/active-channel-card";
import { ChannelAddPanel } from "../views/channels/channel-add-panel";
import { channelLabelKey } from "@supportflow/shared/tauri-bridge/enums";
import { getDevChannel } from "../lib/agent-console/dev-channel";
import { Button } from "@supportflow/ui/button";

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
      const data = await fetchChannels();
      setCatalog(data);
    } catch (e) {
      setCatalog([]);
      setLoadError(e instanceof Error ? e.message : t("channels_load_failed"));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    let cancelled = false;
    queueMicrotask(() => {
      if (!cancelled) void load();
    });
    return () => {
      cancelled = true;
    };
  }, [load]);

  const scopedCatalog = useMemo(() => {
    if (!devChannel) {
      return catalog;
    }
    return catalog.filter((c) => c.name === devChannel);
  }, [catalog, devChannel]);

  const activeChannels = scopedCatalog.filter((c) => c.active);
  const devChannelRow = devChannel ? catalog.find((c) => c.name === devChannel) : undefined;

  const handleDisconnect = async (name: string) => {
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
  };

  const handleConnected = async () => {
    setAddOpen(false);
    await load();
  };

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
          <div className="mb-6 flex items-center justify-between">
            <div>
              <h2 className="text-foreground text-xl font-bold">{t(pageTitleKey)}</h2>
              <p className="text-muted-foreground mt-1 text-sm">{t(pageDescKey)}</p>
            </div>
            {!devChannel ? (
              <Button
                type="button"
                disabled={!!loadError}
                className="flex items-center gap-2"
                onClick={() => setAddOpen(true)}
              >
                <Plus className="size-3.5" />
                {t("channels_add")}
              </Button>
            ) : null}
          </div>

          {loading ? (
            <div className="text-muted-foreground flex items-center justify-center gap-2 py-8 text-sm">
              <Loader2 className="size-4 animate-spin" />
              <span>{t("channels_loading")}</span>
            </div>
          ) : loadError ? (
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
          ) : (
            <>
              <div
                className={`grid gap-4 ${
                  (addOpen || devChannel) && activeChannels.length === 0 ? "hidden" : ""
                }`}
              >
                {activeChannels.length === 0 && !addOpen && !devChannel ? (
                  <div className="flex flex-col items-center justify-center py-20">
                    <div className="bg-info/10 mb-4 flex size-16 items-center justify-center rounded-2xl">
                      <Radio className="text-info size-7" />
                    </div>
                    <p className="text-muted-foreground font-medium">{t("channels_empty")}</p>
                    <p className="text-muted-foreground mt-1 text-sm">{t("channels_empty_desc")}</p>
                  </div>
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

              {(addOpen || (devChannel && activeChannels.length === 0)) && devChannel ? (
                <ChannelAddPanel
                  key={devChannel}
                  catalog={catalog}
                  lang={lang}
                  fixedChannel={devChannel}
                  onClose={() => setAddOpen(false)}
                  onConnected={() => void handleConnected()}
                />
              ) : null}

              {addOpen && !devChannel ? (
                <ChannelAddPanel
                  catalog={catalog}
                  lang={lang}
                  onClose={() => setAddOpen(false)}
                  onConnected={() => void handleConnected()}
                />
              ) : null}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
