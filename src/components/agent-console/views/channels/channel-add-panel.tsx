"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { ChevronDown, Loader2, Plus } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  channelAction,
  localizeChannelText,
  type ChannelCatalogEntry
} from "@/cmd/channel-python-channels";
import { ChannelHint } from "@/components/agent-console/views/channels/channel-hint";
import {
  buildConfigFromDrafts,
  ChannelFields,
  draftsFromChannel,
  type ChannelFieldDrafts
} from "@/components/agent-console/views/channels/channel-fields";
import { FeishuPanel } from "@/components/agent-console/views/channels/feishu-panel";
import { WecomPanel } from "@/components/agent-console/views/channels/wecom-panel";
import { WeixinQrPanel } from "@/components/agent-console/views/channels/weixin-qr-panel";

interface ChannelAddPanelProps {
  catalog: ChannelCatalogEntry[];
  lang: string;
  /** When set (dev preset), skip channel type dropdown. */
  fixedChannel?: string;
  onClose: () => void;
  onConnected: () => void;
}

export function ChannelAddPanel({
  catalog,
  lang,
  fixedChannel,
  onClose,
  onConnected
}: ChannelAddPanelProps) {
  const { t } = useTranslation("console");
  const [selected, setSelected] = useState(fixedChannel ?? "");
  const [open, setOpen] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() => {
    if (!fixedChannel) {
      return { strings: {}, bools: {}, maskedCleared: {} };
    }
    const row = catalog.find((c) => c.name === fixedChannel);
    return row ? draftsFromChannel(row) : { strings: {}, bools: {}, maskedCleared: {} };
  });
  const dropdownRef = useRef<HTMLDivElement>(null);

  const activeNames = new Set(catalog.filter((c) => c.active).map((c) => c.name));
  const available = catalog.filter((c) => !activeNames.has(c.name));
  const selectedChannel = fixedChannel ?? selected;
  const ch = catalog.find((c) => c.name === selectedChannel);

  const pickChannel = useCallback(
    (name: string) => {
      setSelected(name);
      setOpen(false);
      if (!name || name === "weixin") return;
      const next = catalog.find((c) => c.name === name);
      if (next) setDrafts(draftsFromChannel(next));
    },
    [catalog]
  );

  useEffect(() => {
    const onDoc = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, []);

  if (available.length === 0) {
    return (
      <div className="mt-4 rounded-xl border border-slate-200 bg-white p-6 text-center dark:border-white/10 dark:bg-[#1A1A1A]">
        <p className="text-sm text-slate-500 dark:text-slate-400">{t("channels_all_connected")}</p>
        <button
          type="button"
          className="mt-3 cursor-pointer text-xs text-slate-400 hover:text-slate-600 dark:hover:text-slate-300"
          onClick={onClose}
        >
          {t("channels_cancel")}
        </button>
      </div>
    );
  }

  const submitConnect = async () => {
    if (!ch) {
      return;
    }
    setConnecting(true);
    try {
      await channelAction({
        action: "connect",
        channel: ch.name,
        config: buildConfigFromDrafts(ch, drafts)
      });
      onConnected();
      onClose();
    } catch {
      // keep panel open
    } finally {
      setConnecting(false);
    }
  };

  const showActions =
    ch &&
    selectedChannel !== "weixin" &&
    selectedChannel !== "wecom_bot" &&
    selectedChannel !== "feishu";

  const selectLabel =
    selectedChannel && ch
      ? `${localizeChannelText(ch.label, lang)} (${ch.name})`
      : t("channels_select_placeholder");

  return (
    <div className="mt-4 rounded-xl border border-[#35A85B]/30 bg-white p-6 dark:border-[#35A85B]/40 dark:bg-[#1A1A1A]">
      <div className="mb-5 flex items-center gap-3">
        <div className="flex size-9 items-center justify-center rounded-lg bg-[#35A85B]/10 dark:bg-[#35A85B]/20">
          <Plus className="size-4 text-[#35A85B]" />
        </div>
        <h3 className="font-semibold text-slate-800 dark:text-slate-100">{t("channels_add")}</h3>
      </div>

      {fixedChannel ? null : (
        <div className="mb-4">
          <div
            ref={dropdownRef}
            className={`cfg-dropdown ${open ? "open" : ""}`}
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                setOpen(false);
              }
            }}
          >
            <div
              className="cfg-dropdown-selected"
              onClick={() => setOpen((v) => !v)}
              onKeyDown={(e) => e.key === "Enter" && setOpen((v) => !v)}
              role="button"
              tabIndex={0}
            >
              <span className="truncate text-sm">{selectLabel}</span>
              <ChevronDown className="cfg-dropdown-arrow size-3 text-slate-400" />
            </div>
            <div className="cfg-dropdown-menu">
              <div
                className={`cfg-dropdown-item ${!selected ? "active" : ""}`}
                onClick={() => pickChannel("")}
                onKeyDown={() => {}}
                role="option"
                aria-selected={!selected}
              >
                {t("channels_select_placeholder")}
              </div>
              {available.map((item) => (
                <div
                  key={item.name}
                  className={`cfg-dropdown-item ${selected === item.name ? "active" : ""}`}
                  onClick={() => pickChannel(item.name)}
                  role="option"
                  aria-selected={selected === item.name}
                >
                  {localizeChannelText(item.label, lang)} ({item.name})
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      <div className="space-y-4">
        {selectedChannel === "weixin" && ch ? (
          <WeixinQrPanel mode="add" onConnected={onConnected} />
        ) : null}
        {selectedChannel === "wx" && ch ? (
          <>
            {ch.hint ? <ChannelHint hint={ch.hint} lang={lang} /> : null}
            <ChannelFields
              channelName="wx"
              fields={ch.fields}
              lang={lang}
              drafts={drafts}
              onChange={setDrafts}
            />
          </>
        ) : null}
        {selectedChannel === "wecom_bot" && ch ? (
          <WecomPanel
            channel={ch}
            lang={lang}
            variant="add"
            onConnected={onConnected}
            showManualActions
            onManualConnect={onConnected}
          />
        ) : null}
        {selectedChannel === "feishu" && ch ? (
          <FeishuPanel channel={ch} lang={lang} onConnected={onConnected} showConnectButton />
        ) : null}
        {ch &&
        selectedChannel &&
        !["weixin", "wx", "wecom_bot", "feishu"].includes(selectedChannel) ? (
          <>
            {ch.hint ? <ChannelHint hint={ch.hint} lang={lang} /> : null}
            {selectedChannel === "wework" ? (
              <p className="mb-4 text-xs text-slate-500 dark:text-slate-400">
                {t("wework_connect_note")}
              </p>
            ) : null}
            <ChannelFields
              channelName={ch.name}
              fields={ch.fields}
              lang={lang}
              drafts={drafts}
              onChange={setDrafts}
            />
          </>
        ) : null}
      </div>

      {showActions ? (
        <div className="mt-4 flex items-center justify-end gap-3 pt-4">
          <button
            type="button"
            className="cursor-pointer rounded-lg border border-slate-200 px-4 py-2 text-sm font-medium text-slate-600 transition-colors hover:bg-slate-50 dark:border-white/10 dark:text-slate-300 dark:hover:bg-white/5"
            onClick={onClose}
          >
            {t("channels_cancel")}
          </button>
          <button
            type="button"
            disabled={connecting}
            className="flex cursor-pointer items-center rounded-lg bg-[#35A85B] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#228547] disabled:cursor-not-allowed disabled:opacity-50"
            onClick={() => void submitConnect()}
          >
            {connecting ? (
              <>
                <Loader2 className="mr-2 size-4 animate-spin" />
                {t("channels_connecting")}
              </>
            ) : (
              t("channels_connect_btn")
            )}
          </button>
        </div>
      ) : selectedChannel === "weixin" || selectedChannel === "feishu" ? (
        <div className="mt-4 flex justify-end">
          <button
            type="button"
            className="cursor-pointer rounded-lg border border-slate-200 px-4 py-2 text-sm font-medium text-slate-600 dark:border-white/10 dark:text-slate-300"
            onClick={onClose}
          >
            {t("channels_cancel")}
          </button>
        </div>
      ) : null}
    </div>
  );
}
