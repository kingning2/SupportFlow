"use client";

import { useMemo, useState } from "react";
import { Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@supportflow/ui/button";
import {
  channelFieldValueString,
  localizeChannelText,
  type ChannelCatalogEntry,
  type ChannelFieldDrafts
} from "@supportflow/shared";

const WEWORK_DEFAULT_VERSION = "4.0.8.6027";
const WEWORK_DEFAULT_INIT_WAIT_SECONDS = 60;

export interface WeworkConnectPanelProps {
  channel: ChannelCatalogEntry;
  lang: string;
  connecting?: boolean;
  onConnect: (config: Record<string, string | number | boolean>) => void | Promise<void>;
  onCancel?: () => void;
}

function draftsFromWeworkChannel(channel: ChannelCatalogEntry): ChannelFieldDrafts {
  const strings: Record<string, string> = {};
  const bools: Record<string, boolean> = { wework_smart: true };
  for (const field of channel.fields) {
    const raw = channelFieldValueString(field.value);
    if (field.type === "bool" || field.type === "checkbox") {
      bools[field.key] = raw === "true" || raw === "1";
    } else {
      strings[field.key] = raw;
    }
  }
  return { strings, bools, maskedCleared: {} };
}

function buildWeworkConnectConfig(
  drafts: ChannelFieldDrafts
): Record<string, string | number | boolean> {
  const exePath = (drafts.strings.wework_exe_path ?? "").trim();
  return {
    wework_exe_path: exePath,
    wework_smart: drafts.bools.wework_smart ?? true,
    wework_version: WEWORK_DEFAULT_VERSION,
    wework_init_wait_seconds: WEWORK_DEFAULT_INIT_WAIT_SECONDS
  };
}

/** 企微个人号接入表单（仅路径可改，版本/等待时间使用内置默认值） */
export function WeworkConnectPanel({
  channel,
  lang,
  connecting = false,
  onConnect,
  onCancel
}: WeworkConnectPanelProps) {
  const { t } = useTranslation("console");
  const [drafts, setDrafts] = useState<ChannelFieldDrafts>(() => draftsFromWeworkChannel(channel));

  const pathField = useMemo(
    () => channel.fields.find((f) => f.key === "wework_exe_path"),
    [channel.fields]
  );
  const pathLabel = pathField
    ? localizeChannelText(pathField.label, lang)
    : t("wework_connect_path_label");
  const pathPlaceholder = pathField?.placeholder
    ? localizeChannelText(pathField.placeholder, lang)
    : pathLabel;
  const pathValue =
    drafts.strings.wework_exe_path ?? (pathField ? channelFieldValueString(pathField.value) : "");

  return (
    <div className="flex flex-col gap-4">
      <p className="text-xs leading-relaxed text-slate-500">{t("wework_connect_note")}</p>

      <div>
        <label
          htmlFor="wework-exe-path"
          className="mb-1.5 block text-sm font-medium text-[#1A2B4A]"
        >
          {pathLabel}
        </label>
        <input
          id="wework-exe-path"
          type="text"
          value={pathValue}
          placeholder={pathPlaceholder}
          className="w-full rounded-lg border border-[hsl(var(--border))] bg-white px-3 py-2 text-sm text-[#1A2B4A] outline-none focus:border-[var(--wework-blue,#2F7CF6)] focus:ring-1 focus:ring-[var(--wework-blue,#2F7CF6)]/30"
          onChange={(e) =>
            setDrafts((prev) => ({
              ...prev,
              strings: { ...prev.strings, wework_exe_path: e.target.value }
            }))
          }
        />
        <p className="mt-1 text-[10px] text-slate-400">{t("wework_connect_path_hint")}</p>
      </div>

      <label className="flex cursor-pointer items-center gap-2 text-sm text-slate-600">
        <input
          type="checkbox"
          className="size-4 rounded border-slate-300"
          checked={drafts.bools.wework_smart ?? true}
          onChange={(e) =>
            setDrafts((prev) => ({
              ...prev,
              bools: { ...prev.bools, wework_smart: e.target.checked }
            }))
          }
        />
        {t("wework_connect_reuse_client")}
      </label>

      <div className="flex shrink-0 items-center justify-end gap-2 border-t border-[hsl(var(--border))] pt-4">
        {onCancel ? (
          <Button type="button" variant="ghost" disabled={connecting} onClick={onCancel}>
            {t("wework_account_cancel_new")}
          </Button>
        ) : null}
        <Button
          type="button"
          disabled={connecting || !pathValue.trim()}
          className="min-w-22 bg-(--wework-blue) hover:opacity-90"
          onClick={() => void onConnect(buildWeworkConnectConfig(drafts))}
        >
          {connecting ? (
            <>
              <Loader2 className="animate-spin" />
              {t("channels_connecting")}
            </>
          ) : (
            t("channels_connect_btn")
          )}
        </Button>
      </div>
    </div>
  );
}
