"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { Loader2, Plus, Radio } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  channelAction,
  channelLangFromI18n,
  fetchChannels,
  type ChannelCatalogEntry
} from "@/cmd/channel-python-channels";
import { ActiveChannelCard } from "@/components/agent-console/views/channels/active-channel-card";
import { ChannelAddPanel } from "@/components/agent-console/views/channels/channel-add-panel";
import { channelLabelKey } from "@/enums";
import { getDevChannel } from "@/lib/agent-console/dev-channel";

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
              <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100">
                {t(pageTitleKey)}
              </h2>
              <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">{t(pageDescKey)}</p>
            </div>
            {!devChannel ? (
              <button
                type="button"
                disabled={!!loadError}
                className="flex cursor-pointer items-center gap-2 rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#228547] disabled:cursor-not-allowed disabled:opacity-50"
                onClick={() => setAddOpen(true)}
              >
                <Plus className="size-3.5" />
                {t("channels_add")}
              </button>
            ) : null}
          </div>

          {loading ? (
            <div className="flex items-center justify-center gap-2 py-8 text-sm text-slate-400">
              <Loader2 className="size-4 animate-spin" />
              <span>{t("channels_loading")}</span>
            </div>
          ) : loadError ? (
            <div className="rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900 dark:border-amber-900/40 dark:bg-amber-950/30 dark:text-amber-100">
              <p className="font-medium">{t("channels_python_unreachable_title")}</p>
              <p className="mt-2 text-xs opacity-90">{loadError}</p>
              <p className="mt-3 text-xs opacity-80">{t("channels_python_unreachable_hint")}</p>
              <button
                type="button"
                className="mt-4 cursor-pointer rounded-lg border border-amber-300 px-3 py-1.5 text-xs dark:border-amber-800"
                onClick={() => void load()}
              >
                {t("channels_retry")}
              </button>
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
                    <div className="mb-4 flex size-16 items-center justify-center rounded-2xl bg-blue-50 dark:bg-blue-900/20">
                      <Radio className="size-7 text-blue-400" />
                    </div>
                    <p className="font-medium text-slate-500 dark:text-slate-400">
                      {t("channels_empty")}
                    </p>
                    <p className="mt-1 text-sm text-slate-400 dark:text-slate-500">
                      {t("channels_empty_desc")}
                    </p>
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
