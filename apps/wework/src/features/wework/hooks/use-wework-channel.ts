"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import type { ChannelCatalogEntry } from "@supportflow/shared";
import type { ChannelStatusChangedPayload } from "@supportflow/shared/contracts/tauri-payloads";
import { channelLoginStatus } from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";
import { TauriEvent } from "@supportflow/shared/tauri-bridge/enums";
import { tauriOn } from "@supportflow/shared/tauri-bridge/tauri-event";

import type { WeworkConnectionStatus } from "../types/wework-conversation";
import type { WeworkPageActions } from "@/features/wework/accounts/wework-page";

const CONNECTING_PHASES = new Set(["starting", "waiting_login", "logged_in", "syncing"]);

const STATUS_REFRESH_DEBOUNCE_MS = 400;

/** 仅同步后端通道状态，不自动 connect */
export function useWeworkChannel(actions: WeworkPageActions) {
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
    return catalog.find((c) => c.name === "wework") ?? null;
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
      } catch (e) {
        if (seq !== refreshSeq.current) {
          return null;
        }
        setChannel(null);
        setChannelError(e instanceof Error ? e.message : "channels_load_failed");
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
      } catch (e) {
        if (cancelled || seq !== refreshSeq.current) {
          return;
        }
        setChannel(null);
        setChannelError(e instanceof Error ? e.message : "channels_load_failed");
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
      setLifecyclePhase(payload.phase);

      if (payload.phase === "waiting_login" || payload.phase === "starting") {
        setEventLoginStatus("unknown");
        setEventActive(false);
      }

      if (payload.phase === "logged_in") {
        setEventLoginStatus("logged_in");
        setEventActive(false);
        if (payload.userId) {
          setEventLoginProfile({
            user_id: payload.userId,
            display_name: payload.displayName ?? ""
          });
        }
      }

      if (payload.phase === "syncing") {
        setEventLoginStatus("logged_in");
        setEventActive(false);
      }

      if (payload.phase === "ready") {
        setEventLoginStatus("logged_in");
        setEventActive(true);
      }

      if (payload.phase === "error") {
        setEventActive(false);
      }

      if (statusDebounceRef.current) {
        clearTimeout(statusDebounceRef.current);
      }
      statusDebounceRef.current = setTimeout(() => {
        statusDebounceRef.current = null;
        void refreshChannel({ silent: true });
      }, STATUS_REFRESH_DEBOUNCE_MS);
    });
  }, [refreshChannel]);

  const connectionStatus: WeworkConnectionStatus = (() => {
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
    const login = channel ? channelLoginStatus(channel) : undefined;
    if (login === "logged_in") {
      return "connecting";
    }
    return "disconnected";
  })();

  const effectiveChannel: ChannelCatalogEntry | null = (() => {
    if (!channel && !eventLoginStatus && !eventLoginProfile) {
      return null;
    }

    return {
      name: channel?.name ?? "wework",
      label: channel?.label ?? { zh: "企业微信个人号", en: "WeCom Desktop" },
      active: channel?.active ?? eventActive,
      fields: channel?.fields ?? [],
      hint: channel?.hint,
      icon: channel?.icon,
      color: channel?.color,
      login_status: channel?.login_status ?? channel?.loginStatus ?? eventLoginStatus ?? undefined,
      loginStatus: channel?.loginStatus ?? channel?.login_status ?? eventLoginStatus ?? undefined,
      login_profile: channel?.login_profile ?? eventLoginProfile ?? undefined
    };
  })();

  return {
    channel: effectiveChannel,
    channelLoading,
    channelError,
    connectionStatus,
    refreshChannel
  };
}
