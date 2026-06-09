"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { ChevronDown, Loader2, Plus } from "lucide-react";

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
import { WeworkConnectPanel } from "./wework-connect-panel";
import { Button } from "@supportflow/ui/button";

interface ChannelAddPanelProps {
  catalog: ChannelCatalogEntry[];
  lang: string;
  /** When set (dev preset), skip channel type dropdown. */
  fixedChannel?: string;
  onClose: () => void;
  onConnected: () => void;
}

function emptyDrafts(): ChannelFieldDrafts {
  return { strings: {}, bools: {}, maskedCleared: {} };
}

function resolveDrafts(catalog: ChannelCatalogEntry[], channelName?: string): ChannelFieldDrafts {
  if (!channelName) {
    return emptyDrafts();
  }
  const row = catalog.find((channel) => channel.name === channelName);
  return row ? draftsFromChannel(row) : emptyDrafts();
}

export function ChannelAddPanel({
  catalog,
  lang,
  fixedChannel,
  onClose,
  onConnected
}: ChannelAddPanelProps) {
  const [selected, setSelected] = useState(fixedChannel ?? "");
  const [open, setOpen] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() =>
    resolveDrafts(catalog, fixedChannel)
  );
  const dropdownRef = useRef<HTMLDivElement>(null);

  const activeNames = new Set(catalog.filter((c) => c.active).map((c) => c.name));
  const available = catalog.filter((c) => !activeNames.has(c.name));
  const selectedChannel = fixedChannel ?? selected;
  const ch = catalog.find((c) => c.name === selectedChannel);

  const pickChannel = useCallback(
    (name: string) => {
      setSelected(name);
      setOpen(false);
      setDrafts(resolveDrafts(catalog, name));
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
      <div className="bg-card border-border mt-4 rounded-xl border p-6 text-center">
        <p className="text-muted-foreground text-sm">{"所有可用通道均已接入"}</p>
        <Button
          type="button"
          variant="ghost"
          className="text-muted-foreground mt-3 text-xs"
          onClick={onClose}
        >
          {"取消"}
        </Button>
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

  const showActions = ch && selectedChannel === "wx";
  const showWxPanel = selectedChannel === "wx" && ch;
  const showWeworkPanel = selectedChannel === "wework" && ch;

  const selectLabel =
    selectedChannel && ch
      ? `${localizeChannelText(ch.label, lang)} (${ch.name})`
      : "选择要接入的通道…";

  return (
    <div className="border-primary/30 bg-card mt-4 rounded-xl border p-6">
      <div className="mb-5 flex items-center gap-3">
        <div className="bg-primary/10 flex size-9 items-center justify-center rounded-lg">
          <Plus className="text-primary size-4" />
        </div>
        <h3 className="text-foreground font-semibold">{"接入通道"}</h3>
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
              <ChevronDown className="cfg-dropdown-arrow text-muted-foreground size-3" />
            </div>
            <div className="cfg-dropdown-menu">
              <div
                className={`cfg-dropdown-item ${!selected ? "active" : ""}`}
                onClick={() => pickChannel("")}
                onKeyDown={() => {}}
                role="option"
                aria-selected={!selected}
              >
                {"选择要接入的通道…"}
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
        {showWxPanel ? (
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
        {showWeworkPanel ? (
          <WeworkConnectPanel
            channel={ch}
            lang={lang}
            connecting={connecting}
            onConnect={async (config) => {
              setConnecting(true);
              try {
                await channelAction({ action: "connect", channel: "wework", config });
                onConnected();
                onClose();
              } catch {
                // keep panel open
              } finally {
                setConnecting(false);
              }
            }}
          />
        ) : null}
      </div>

      {showActions ? (
        <div className="mt-4 flex items-center justify-end gap-3 pt-4">
          <Button type="button" variant="outline" onClick={onClose}>
            {"取消"}
          </Button>
          <Button
            type="button"
            disabled={connecting}
            className="flex items-center"
            onClick={() => void submitConnect()}
          >
            {connecting ? (
              <>
                <Loader2 className="mr-2 size-4 animate-spin" />
                {"接入中…"}
              </>
            ) : (
              "接入"
            )}
          </Button>
        </div>
      ) : null}
    </div>
  );
}
