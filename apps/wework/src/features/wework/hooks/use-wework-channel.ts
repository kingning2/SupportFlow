"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import type { ChannelCatalogEntry } from "@supportflow/shared";
import type { ChannelStatusChangedPayload } from "@supportflow/shared/contracts/tauri-payloads";
import { channelLoginStatus } from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import { TauriEvent } from "@supportflow/shared/tauri-bridge/enums";
import { tauriOn } from "@supportflow/shared/tauri-bridge/tauri-event";

import type { WeworkConnectionStatus } from "../types/wework-conversation";
import type { PageActions } from "@/features/wework/accounts/page-types";

const CONNECTING_PHASES = new Set(["starting", "waiting_login", "logged_in", "syncing"]);
const STATUS_REFRESH_DEBOUNCE_MS = 400;

function toChannelLoadError(error: unknown): string {
  return error instanceof Error ? error.message : "channels_load_failed";
}

function fallbackWeworkLabel() {
  return { zh: "WeCom", en: "WeCom Desktop" };
}

function resolveEffectiveChannelName(channel: ChannelCatalogEntry | null) {
  return channel?.name ?? "wework";
}

function resolveEffectiveChannelLabel(channel: ChannelCatalogEntry | null) {
  return channel?.label ?? fallbackWeworkLabel();
}

function resolveEffectiveChannelActive(channel: ChannelCatalogEntry | null, eventActive: boolean) {
  return channel?.active ?? eventActive;
}

function resolveEffectiveChannelFields(channel: ChannelCatalogEntry | null) {
  return channel?.fields ?? [];
}

function resolveEffectiveLoginStatus(
  channel: ChannelCatalogEntry | null,
  eventLoginStatus: string | null
) {
  return channel?.login_status ?? channel?.loginStatus ?? eventLoginStatus ?? undefined;
}

function buildEffectiveChannel(
  channel: ChannelCatalogEntry | null,
  eventLoginStatus: string | null,
  eventLoginProfile: { user_id: string; display_name: string } | null,
  eventActive: boolean
): ChannelCatalogEntry | null {
  if (!channel && !eventLoginStatus && !eventLoginProfile) {
    return null;
  }

  return {
    name: resolveEffectiveChannelName(channel),
    label: resolveEffectiveChannelLabel(channel),
    active: resolveEffectiveChannelActive(channel, eventActive),
    fields: resolveEffectiveChannelFields(channel),
    hint: channel?.hint,
    icon: channel?.icon,
    color: channel?.color,
    login_status: resolveEffectiveLoginStatus(channel, eventLoginStatus),
    loginStatus: resolveEffectiveLoginStatus(channel, eventLoginStatus),
    login_profile: channel?.login_profile ?? eventLoginProfile ?? undefined
  };
}

function deriveConnectionStatus(params: {
  channel: ChannelCatalogEntry | null;
  channelLoading: boolean;
  eventActive: boolean;
  lifecyclePhase: string | null;
}): WeworkConnectionStatus {
  const { channel, channelLoading, eventActive, lifecyclePhase } = params;
  if (eventActive) {
    return "ready";
  }
  if (channelLoading) {
    return "connecting";
  }
  if (channel?.active) {
    return "ready";
  }
  if (lifecyclePhase && CONNECTING_PHASES.has(lifecyclePhase)) {
    return "connecting";
  }

  return channel && channelLoginStatus(channel) === "logged_in" ? "connecting" : "disconnected";
}

function applyLifecycleEvent(
  payload: ChannelStatusChangedPayload,
  setLifecyclePhase: (phase: string) => void,
  setEventLoginStatus: (status: string) => void,
  setEventActive: (active: boolean) => void,
  setEventLoginProfile: (profile: { user_id: string; display_name: string }) => void
) {
  setLifecyclePhase(payload.phase);

  switch (payload.phase) {
    case "waiting_login":
    case "starting":
      setEventLoginStatus("unknown");
      setEventActive(false);
      return;
    case "logged_in":
      setEventLoginStatus("logged_in");
      setEventActive(false);
      if (payload.userId) {
        setEventLoginProfile({
          user_id: payload.userId,
          display_name: payload.displayName ?? ""
        });
      }
      return;
    case "syncing":
      setEventLoginStatus("logged_in");
      setEventActive(false);
      return;
    case "ready":
      setEventLoginStatus("logged_in");
      setEventActive(true);
      return;
    case "error":
      setEventActive(false);
      return;
    default:
      return;
  }
}

/** 同步后端通道状态，不自动 connect */
export function useWeworkChannel(actions: PageActions) {
  const [channel, setChannel] = useState<ChannelCatalogEntry | null>(null);
  const [channelLoading, setChannelLoading] = useState(true);
  const [channelError, setChannelError] = useState<string | null>(null);
  const [lifecyclePhase, setLifecyclePhase] = useState<string | null>(null);
  const [eventLoginProfile, setEventLoginProfile] = useState<{
    user_id: string;
    display_name: string;
  } | null>(null);
  const [eventLoginStatus, setEventLoginStatus] = useState<string | null>(null);
  const [eventActive, setEventActive] = useState(false);

  const refreshSeq = useRef(0);
  const statusDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const fetchWeworkChannel = useCallback(async (): Promise<ChannelCatalogEntry | null> => {
    const catalog = await actions.fetchChannels();
    return catalog.find((entry) => entry.name === "wework") ?? null;
  }, [actions]);

  const refreshChannel = useCallback(
    async (options?: { silent?: boolean }) => {
      const silent = options?.silent ?? false;
      const seq = ++refreshSeq.current;

      if (!silent) {
        setChannelLoading(true);
      }

      setChannelError(null);
      try {
        const row = await fetchWeworkChannel();
        if (seq !== refreshSeq.current) {
          return row;
        }
        setChannel(row);
        return row;
      } catch (error) {
        if (seq !== refreshSeq.current) {
          return null;
        }
        setChannel(null);
        setChannelError(toChannelLoadError(error));
        return null;
      } finally {
        if (!silent && seq === refreshSeq.current) {
          setChannelLoading(false);
        }
      }
    },
    [fetchWeworkChannel]
  );

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const seq = ++refreshSeq.current;
      setChannelLoading(true);
      setChannelError(null);
      try {
        const row = await fetchWeworkChannel();
        if (cancelled || seq !== refreshSeq.current) {
          return;
        }
        setChannel(row);
      } catch (error) {
        if (cancelled || seq !== refreshSeq.current) {
          return;
        }
        setChannel(null);
        setChannelError(toChannelLoadError(error));
      } finally {
        if (!cancelled && seq === refreshSeq.current) {
          setChannelLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [fetchWeworkChannel]);

  useEffect(() => {
    return () => {
      if (statusDebounceRef.current) {
        clearTimeout(statusDebounceRef.current);
      }
    };
  }, []);

  useEffect(() => {
    return tauriOn<ChannelStatusChangedPayload>(TauriEvent.ChannelStatusChanged, (event) => {
      const payload = event.payload;
      if (payload.channel !== "wework") {
        return;
      }

      applyLifecycleEvent(
        payload,
        setLifecyclePhase,
        setEventLoginStatus,
        setEventActive,
        setEventLoginProfile
      );

      if (statusDebounceRef.current) {
        clearTimeout(statusDebounceRef.current);
      }
      statusDebounceRef.current = setTimeout(() => {
        statusDebounceRef.current = null;
        void refreshChannel({ silent: true });
      }, STATUS_REFRESH_DEBOUNCE_MS);
    });
  }, [refreshChannel]);

  const connectionStatus = deriveConnectionStatus({
    channel,
    channelLoading,
    eventActive,
    lifecyclePhase
  });

  const effectiveChannel = buildEffectiveChannel(
    channel,
    eventLoginStatus,
    eventLoginProfile,
    eventActive
  );

  return {
    channel: effectiveChannel,
    channelLoading,
    channelError,
    connectionStatus,
    refreshChannel
  };
}
