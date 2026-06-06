"use client";

import type { Dispatch, SetStateAction } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";

import {
  buildConfigFromDrafts,
  draftsFromChannel,
  type ChannelCatalogEntry
} from "@supportflow/shared";

import {
  loadActiveAccountId,
  loadSavedAccounts,
  persistActiveAccountId,
  removeSavedAccount,
  resolveWeworkLoginFromCatalog,
  upsertSavedAccount
} from "@/features/wework/accounts/storage";
import type { WeworkAccountConfig, WeworkSavedAccount } from "@/features/wework/accounts/types";
import type { WeworkPageActions } from "@/features/wework/accounts/wework-page-types";
import type { WeworkConnectionStatus } from "@/features/wework/types/wework-conversation";

type Translate = (key: string, options?: Record<string, unknown>) => string;

export interface WeworkPageState {
  accounts: WeworkSavedAccount[];
  accountsLoaded: boolean;
  activeAccount: WeworkSavedAccount | null;
  activeAccountId: string | null;
  backendErrorMessage: string | null;
  backendReady: boolean;
  channelActive: boolean;
  connectingId: string | null;
  disconnecting: boolean;
  menuOpenId: string | null;
  newConnecting: boolean;
  showNewForm: boolean;
  switchTarget: WeworkSavedAccount | null;
  switching: boolean;
  syncingContacts: boolean;
}

export interface WeworkPageHandlers {
  handleAccountClick: (account: WeworkSavedAccount) => void;
  handleConfirmSwitch: () => Promise<void>;
  handleDeleteAccount: (id: string) => void;
  handleDisconnect: () => Promise<void>;
  handleNewConnect: (config: Record<string, string | number | boolean>) => Promise<void>;
  handleSyncContacts: () => Promise<void>;
  handleSwitchOpenChange: (open: boolean) => void;
  onRetry: () => void;
  setMenuOpenId: Dispatch<SetStateAction<string | null>>;
  setShowNewForm: Dispatch<SetStateAction<boolean>>;
}

function toAccountConfig(config: Record<string, string | number | boolean>): WeworkAccountConfig {
  return {
    wework_exe_path:
      typeof config.wework_exe_path === "string" ? config.wework_exe_path : undefined,
    wework_version: typeof config.wework_version === "string" ? config.wework_version : undefined,
    wework_smart: typeof config.wework_smart === "boolean" ? config.wework_smart : undefined,
    wework_init_wait_seconds:
      typeof config.wework_init_wait_seconds === "number"
        ? config.wework_init_wait_seconds
        : undefined
  };
}

function resolveActiveAccount(params: {
  accounts: WeworkSavedAccount[];
  activeAccountId: string | null;
  channel: ChannelCatalogEntry | null;
  channelActive: boolean;
}) {
  const { accounts, activeAccountId, channel, channelActive } = params;
  if (!channelActive) {
    return null;
  }

  return (
    accounts.find((account) => account.id === activeAccountId) ??
    accounts.find(
      (account) => account.weworkUserId && channel?.login_profile?.user_id === account.weworkUserId
    ) ??
    accounts[0] ??
    null
  );
}

export function useWeworkPageState(params: {
  actions: WeworkPageActions;
  channel: ChannelCatalogEntry | null;
  channelError: string | null;
  channelLoading: boolean;
  connectionStatus: WeworkConnectionStatus;
  onChannelUpdated?: () => void;
  t: Translate;
}) {
  const { actions, channel, channelError, channelLoading, connectionStatus, onChannelUpdated, t } =
    params;
  const [accounts, setAccounts] = useState<WeworkSavedAccount[]>([]);
  const [accountsLoaded, setAccountsLoaded] = useState(false);
  const [storedActiveAccountId, setStoredActiveAccountId] = useState<string | null>(null);
  const [connectingId, setConnectingId] = useState<string | null>(null);
  const [disconnecting, setDisconnecting] = useState(false);
  const [showNewForm, setShowNewForm] = useState(false);
  const [newConnecting, setNewConnecting] = useState(false);
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);
  const [switchTarget, setSwitchTarget] = useState<WeworkSavedAccount | null>(null);
  const [switching, setSwitching] = useState(false);
  const [syncingContacts, setSyncingContacts] = useState(false);

  const backendReady = !channelLoading && !channelError;
  const backendErrorMessage =
    channelError === "channels_load_failed" ? t("channels_load_failed") : channelError;
  const activeAccountId =
    channelLoading || channelError || !channel?.active ? null : storedActiveAccountId;
  const channelActive =
    backendReady &&
    (connectionStatus === "ready" ||
      resolveWeworkLoginFromCatalog(channel).weworkUserId !== undefined);
  const activeAccount = useMemo(
    () =>
      resolveActiveAccount({
        accounts,
        activeAccountId,
        channel,
        channelActive
      }),
    [accounts, activeAccountId, channel, channelActive]
  );

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const [saved, activeId] = await Promise.all([loadSavedAccounts(), loadActiveAccountId()]);
        if (cancelled) {
          return;
        }
        setAccounts(saved);
        setStoredActiveAccountId(activeId);
      } finally {
        if (!cancelled) {
          setAccountsLoaded(true);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (channelLoading || channelError || !channel?.active) {
      return;
    }
    void loadActiveAccountId().then(setStoredActiveAccountId);
  }, [channel?.active, channelError, channelLoading]);

  useEffect(() => {
    if (
      !accountsLoaded ||
      channelLoading ||
      channelError ||
      !channel?.active ||
      accounts.length > 0
    ) {
      return;
    }

    const config = toAccountConfig(buildConfigFromDrafts(channel, draftsFromChannel(channel)));
    if (!config.wework_exe_path) {
      return;
    }

    const login = resolveWeworkLoginFromCatalog(channel);
    void (async () => {
      const { accounts: next, account } = await upsertSavedAccount(
        [],
        config,
        login.label,
        login.weworkUserId
      );
      await persistActiveAccountId(account.id);
      setAccounts(next);
      setStoredActiveAccountId(account.id);
    })();
  }, [accounts.length, accountsLoaded, channel, channelError, channelLoading]);

  const notifyChannelUpdated = useCallback(() => {
    onChannelUpdated?.();
  }, [onChannelUpdated]);

  const persistAfterConnect = useCallback(
    async (config: WeworkAccountConfig, fallbackLabel?: string) => {
      const catalog = await actions.fetchChannels();
      const row = catalog.find((entry) => entry.name === "wework");
      const login = resolveWeworkLoginFromCatalog(row);
      const current = await loadSavedAccounts();
      const { accounts: next, account } = await upsertSavedAccount(
        current,
        config,
        login.label ?? fallbackLabel,
        login.weworkUserId
      );
      await persistActiveAccountId(account.id);
      setAccounts(next);
      setStoredActiveAccountId(account.id);
    },
    [actions]
  );

  const performDisconnect = useCallback(async () => {
    await actions.disconnect();
    await persistActiveAccountId(null);
    setStoredActiveAccountId(null);
    notifyChannelUpdated();
  }, [actions, notifyChannelUpdated]);

  const connectAccount = useCallback(
    async (account: WeworkSavedAccount) => {
      setConnectingId(account.id);
      try {
        await actions.connect(account.config as Record<string, string | number | boolean>);
        await persistAfterConnect(account.config, account.label);
        notifyChannelUpdated();
      } finally {
        setConnectingId(null);
      }
    },
    [actions, notifyChannelUpdated, persistAfterConnect]
  );

  const handleAccountClick = useCallback(
    (account: WeworkSavedAccount) => {
      if (!backendReady || connectingId || disconnecting || switching) {
        return;
      }
      if (channelActive && activeAccountId === account.id) {
        return;
      }
      if (channelActive) {
        setSwitchTarget(account);
        return;
      }
      void connectAccount(account);
    },
    [
      activeAccountId,
      backendReady,
      channelActive,
      connectAccount,
      connectingId,
      disconnecting,
      switching
    ]
  );

  const handleConfirmSwitch = useCallback(async () => {
    if (!switchTarget) {
      return;
    }
    const target = switchTarget;
    setSwitching(true);
    try {
      await performDisconnect();
      setSwitchTarget(null);
      await connectAccount(target);
    } finally {
      setSwitching(false);
    }
  }, [connectAccount, performDisconnect, switchTarget]);

  const handleNewConnect = useCallback(
    async (config: Record<string, string | number | boolean>) => {
      setNewConnecting(true);
      try {
        await actions.connect(config);
        await persistAfterConnect(toAccountConfig(config));
        setShowNewForm(false);
        notifyChannelUpdated();
      } finally {
        setNewConnecting(false);
      }
    },
    [actions, notifyChannelUpdated, persistAfterConnect]
  );

  const handleSyncContacts = useCallback(async () => {
    setSyncingContacts(true);
    try {
      await actions.syncContacts();
      setAccounts(await loadSavedAccounts());
      notifyChannelUpdated();
    } finally {
      setSyncingContacts(false);
    }
  }, [actions, notifyChannelUpdated]);

  const handleDisconnect = useCallback(async () => {
    if (!window.confirm(t("channels_disconnect_confirm"))) {
      return;
    }
    setDisconnecting(true);
    try {
      await performDisconnect();
    } finally {
      setDisconnecting(false);
    }
  }, [performDisconnect, t]);

  const handleDeleteAccount = useCallback(
    (id: string) => {
      if (!window.confirm(t("wework_account_delete_confirm"))) {
        return;
      }
      void (async () => {
        const next = await removeSavedAccount(accounts, id);
        setAccounts(next);
        if (activeAccountId === id) {
          await persistActiveAccountId(null);
          setStoredActiveAccountId(null);
        }
        setMenuOpenId(null);
      })();
    },
    [accounts, activeAccountId, t]
  );

  const handleSwitchOpenChange = useCallback(
    (open: boolean) => {
      if (!open && !switching) {
        setSwitchTarget(null);
      }
    },
    [switching]
  );

  return {
    handlers: {
      handleAccountClick,
      handleConfirmSwitch,
      handleDeleteAccount,
      handleDisconnect,
      handleNewConnect,
      handleSyncContacts,
      handleSwitchOpenChange,
      onRetry: notifyChannelUpdated,
      setMenuOpenId,
      setShowNewForm
    } satisfies WeworkPageHandlers,
    state: {
      accounts,
      accountsLoaded,
      activeAccount,
      activeAccountId,
      backendErrorMessage,
      backendReady,
      channelActive,
      connectingId,
      disconnecting,
      menuOpenId,
      newConnecting,
      showNewForm,
      switchTarget,
      switching,
      syncingContacts
    } satisfies WeworkPageState
  };
}
