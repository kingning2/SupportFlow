"use client";

import { useCallback, useEffect, useState } from "react";
import { Loader2, Plus, Radio } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  cowChannelAction,
  cowLangFromI18n,
  fetchCowChannels,
  type CowChannel
} from "@/cmd/cow-python-channels";
import { ActiveChannelCard } from "@/components/agent-console/views/channels/active-channel-card";
import { ChannelAddPanel } from "@/components/agent-console/views/channels/channel-add-panel";

export function ChannelsView() {
  const { t, i18n } = useTranslation("console");
  const lang = cowLangFromI18n(i18n.language);
  const [loading, setLoading] = useState(true);
  const [catalog, setCatalog] = useState<CowChannel[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [addOpen, setAddOpen] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const data = await fetchCowChannels();
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

  const activeChannels = catalog.filter((c) => c.active);

  const handleDisconnect = async (name: string) => {
    if (!window.confirm(t("channels_disconnect_confirm"))) {
      return;
    }
    try {
      await cowChannelAction({ action: "disconnect", channel: name });
      await load();
    } catch {
      // noop
    }
  };

  const handleConnected = async () => {
    setAddOpen(false);
    await load();
  };

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-4xl">
          <div className="mb-6 flex items-center justify-between">
            <div>
              <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100">
                {t("channels_title")}
              </h2>
              <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
                {t("channels_desc")}
              </p>
            </div>
            <button
              type="button"
              disabled={!!loadError}
              className="flex cursor-pointer items-center gap-2 rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#228547] disabled:cursor-not-allowed disabled:opacity-50"
              onClick={() => setAddOpen(true)}
            >
              <Plus className="size-3.5" />
              {t("channels_add")}
            </button>
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
                className={`grid gap-4 ${addOpen && activeChannels.length === 0 ? "hidden" : ""}`}
              >
                {activeChannels.length === 0 && !addOpen ? (
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

              {addOpen ? (
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
