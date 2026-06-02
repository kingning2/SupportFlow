"use client";

import { useCallback, useEffect, useState } from "react";
import { Building2, Loader2, Radio } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  buildConfigFromDrafts,
  ChannelFields,
  ChannelHint,
  draftsFromChannel,
  localizeChannelText,
  type ChannelCatalogEntry,
  type ChannelFieldDrafts
} from "@supportflow/shared";

import { WeworkConnectPanel } from "@supportflow/ui/channel/wework-connect-panel";

export interface WeworkPageActions {
  fetchChannels: () => Promise<ChannelCatalogEntry[]>;
  connect: (config: Record<string, string | number | boolean>) => Promise<void>;
  disconnect: () => Promise<void>;
  save: (config: Record<string, string | number | boolean>) => Promise<void>;
}

export interface WeworkPageProps {
  lang: string;
  actions: WeworkPageActions;
}

/** 企微个人号独立入口页 */
export function WeworkPage({ lang, actions }: WeworkPageProps) {
  const { t } = useTranslation("console");
  const [loading, setLoading] = useState(true);
  const [channel, setChannel] = useState<ChannelCatalogEntry | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [disconnecting, setDisconnecting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>({
    strings: {},
    bools: {},
    maskedCleared: {}
  });

  const load = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const catalog = await actions.fetchChannels();
      const row = catalog.find((c) => c.name === "wework") ?? null;
      setChannel(row);
      if (row) {
        setDrafts(draftsFromChannel(row));
      }
    } catch (e) {
      setChannel(null);
      setLoadError(e instanceof Error ? e.message : t("channels_load_failed"));
    } finally {
      setLoading(false);
    }
  }, [actions, t]);

  useEffect(() => {
    let cancelled = false;
    queueMicrotask(() => {
      if (!cancelled) void load();
    });
    return () => {
      cancelled = true;
    };
  }, [load]);

  const handleConnect = async (config: Record<string, string | number | boolean>) => {
    setConnecting(true);
    try {
      await actions.connect(config);
      await load();
    } catch {
      // keep form open
    } finally {
      setConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    if (!window.confirm(t("channels_disconnect_confirm"))) {
      return;
    }
    setDisconnecting(true);
    try {
      await actions.disconnect();
      await load();
    } catch {
      // noop
    } finally {
      setDisconnecting(false);
    }
  };

  const handleSave = async () => {
    if (!channel) {
      return;
    }
    setSaving(true);
    try {
      await actions.save(buildConfigFromDrafts(channel, drafts));
      await load();
    } catch {
      // noop
    } finally {
      setSaving(false);
    }
  };

  const label = channel ? localizeChannelText(channel.label, lang) : t("channel_label_wework");

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-2xl">
          <div className="mb-8 flex items-start gap-4">
            <div className="flex size-12 shrink-0 items-center justify-center rounded-2xl bg-emerald-50 dark:bg-emerald-900/20">
              <Building2 className="size-6 text-emerald-600 dark:text-emerald-400" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-slate-800 dark:text-slate-100">{label}</h1>
              <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
                {t("channels_dev_desc")}
              </p>
            </div>
          </div>

          {loading ? (
            <div className="flex items-center justify-center gap-2 py-16 text-sm text-slate-400">
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
          ) : !channel ? (
            <div className="rounded-xl border border-red-200 bg-red-50 p-4 text-sm text-red-800 dark:border-red-900/40 dark:bg-red-950/30 dark:text-red-100">
              {t("channels_dev_unknown", { channel: "wework" })}
            </div>
          ) : channel.active ? (
            <div className="rounded-xl border border-slate-200 bg-white p-6 dark:border-white/10 dark:bg-[#1A1A1A]">
              <div className="mb-6 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="size-2 rounded-full bg-[#4ABE6E]" />
                  <span className="text-sm font-medium text-[#35A85B]">
                    {t("channels_connected")}
                  </span>
                </div>
                <button
                  type="button"
                  disabled={disconnecting}
                  className="cursor-pointer rounded-lg border border-red-200 px-3 py-1.5 text-xs font-medium text-red-600 transition-colors hover:bg-red-50 disabled:opacity-50 dark:border-red-900/40 dark:text-red-400 dark:hover:bg-red-950/30"
                  onClick={() => void handleDisconnect()}
                >
                  {disconnecting ? t("channels_connecting") : t("channels_disconnect")}
                </button>
              </div>

              {channel.hint ? <ChannelHint hint={channel.hint} lang={lang} /> : null}
              <ChannelFields
                channelName="wework"
                fields={channel.fields}
                lang={lang}
                drafts={drafts}
                onChange={setDrafts}
              />
              <div className="mt-4 flex justify-end">
                <button
                  type="button"
                  disabled={saving}
                  className="flex cursor-pointer items-center rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white hover:bg-[#228547] disabled:opacity-50"
                  onClick={() => void handleSave()}
                >
                  {saving ? (
                    <>
                      <Loader2 className="mr-2 size-4 animate-spin" />
                      {t("channels_saving")}
                    </>
                  ) : (
                    t("channels_save")
                  )}
                </button>
              </div>
            </div>
          ) : (
            <div className="rounded-xl border border-[#35A85B]/30 bg-white p-6 dark:border-[#35A85B]/40 dark:bg-[#1A1A1A]">
              <div className="mb-5 flex items-center gap-3">
                <div className="flex size-9 items-center justify-center rounded-lg bg-[#35A85B]/10 dark:bg-[#35A85B]/20">
                  <Radio className="size-4 text-[#35A85B]" />
                </div>
                <h2 className="font-semibold text-slate-800 dark:text-slate-100">
                  {t("channels_add")}
                </h2>
              </div>
              <WeworkConnectPanel
                channel={channel}
                lang={lang}
                connecting={connecting}
                onConnect={handleConnect}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
