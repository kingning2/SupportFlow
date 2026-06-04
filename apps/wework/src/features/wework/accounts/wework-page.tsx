"use client";

import { useCallback, useEffect, useState } from "react";
import { Building2, Loader2, MoreHorizontal, Plus, Radio, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  buildConfigFromDrafts,
  cn,
  draftsFromChannel,
  type ChannelCatalogEntry
} from "@supportflow/shared";

import { Button } from "@supportflow/ui/button";
import { WeworkConnectPanel } from "@supportflow/ui/channel/wework-connect-panel";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from "@supportflow/ui/dialog";

import { AccountAvatar } from "@/features/wework/accounts/account-avatar";
import {
  loadActiveAccountId,
  loadSavedAccounts,
  persistActiveAccountId,
  removeSavedAccount,
  resolveWeworkLoginFromCatalog,
  upsertSavedAccount
} from "@/features/wework/accounts/storage";
import type { WeworkAccountConfig, WeworkSavedAccount } from "@/features/wework/accounts/types";
import type { WeworkConnectionStatus } from "@/features/wework/types/wework-conversation";

export interface WeworkPageActions {
  fetchChannels: () => Promise<ChannelCatalogEntry[]>;
  connect: (config: Record<string, string | number | boolean>) => Promise<void>;
  disconnect: () => Promise<void>;
  save: (config: Record<string, string | number | boolean>) => Promise<void>;
}

export interface WeworkPageProps {
  lang: string;
  actions: WeworkPageActions;
  /** 与侧栏 useWeworkChannel 共用，避免「已连接」与后端状态不一致 */
  channel: ChannelCatalogEntry | null;
  channelLoading: boolean;
  channelError: string | null;
  connectionStatus: WeworkConnectionStatus;
  onChannelUpdated?: () => void;
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

export function WeworkPage({
  lang,
  actions,
  channel,
  channelLoading,
  channelError,
  connectionStatus,
  onChannelUpdated
}: WeworkPageProps) {
  const { t } = useTranslation("console");
  const [accounts, setAccounts] = useState<WeworkSavedAccount[]>([]);
  const [accountsLoaded, setAccountsLoaded] = useState(false);
  const [storedActiveAccountId, setStoredActiveAccountId] = useState<string | null>(null);
  const activeAccountId =
    channelLoading || channelError || !channel?.active ? null : storedActiveAccountId;
  const [connectingId, setConnectingId] = useState<string | null>(null);
  const [disconnecting, setDisconnecting] = useState(false);
  const [showNewForm, setShowNewForm] = useState(false);
  const [newConnecting, setNewConnecting] = useState(false);
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);
  const [switchTarget, setSwitchTarget] = useState<WeworkSavedAccount | null>(null);
  const [switching, setSwitching] = useState(false);

  const backendReady = !channelLoading && !channelError;
  const backendErrorMessage =
    channelError === "channels_load_failed" ? t("channels_load_failed") : channelError;

  /** 仅在后端确认通道 active 且非加载中时展示「已连接」 */
  const channelActive = backendReady && connectionStatus === "ready" && Boolean(channel?.active);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const [saved, activeId] = await Promise.all([loadSavedAccounts(), loadActiveAccountId()]);
        if (!cancelled) {
          setAccounts(saved);
          setStoredActiveAccountId(activeId);
          setAccountsLoaded(true);
        }
      } catch {
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
  }, [channel?.active, channelLoading, channelError]);

  useEffect(() => {
    if (!accountsLoaded || channelLoading || channelError || !channel?.active) {
      return;
    }
    if (accounts.length > 0) {
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
  }, [accounts.length, accountsLoaded, channel, channelLoading, channelError]);

  const activeAccount = channelActive
    ? (accounts.find((a) => a.id === activeAccountId) ??
      accounts.find((a) => a.weworkUserId && channel?.login_profile?.user_id === a.weworkUserId) ??
      accounts[0] ??
      null)
    : null;

  const persistAfterConnect = useCallback(
    async (config: WeworkAccountConfig, fallbackLabel?: string) => {
      const catalog = await actions.fetchChannels();
      const row = catalog.find((c) => c.name === "wework");
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

  const waitUntilActive = useCallback(async () => {
    // wework_init_wait_seconds defaults to 60; allow headroom for contact sync
    for (let i = 0; i < 60; i++) {
      try {
        const catalog = await actions.fetchChannels();
        const row = catalog.find((c) => c.name === "wework");
        if (row?.active) {
          return true;
        }
      } catch {
        // catalog RPC may fail while sidecar is starting/syncing
      }
      await new Promise((r) => setTimeout(r, 2000));
    }
    return false;
  }, [actions]);

  const performDisconnect = useCallback(async () => {
    await actions.disconnect();
    await persistActiveAccountId(null);
    setStoredActiveAccountId(null);
    onChannelUpdated?.();
  }, [actions, onChannelUpdated]);

  const connectAccount = useCallback(
    async (account: WeworkSavedAccount) => {
      setConnectingId(account.id);
      try {
        await actions.connect(account.config as Record<string, string | number | boolean>);
        const ok = await waitUntilActive();
        if (ok) {
          await persistAfterConnect(account.config, account.label);
          onChannelUpdated?.();
        }
      } catch {
        // keep list
      } finally {
        setConnectingId(null);
      }
    },
    [actions, onChannelUpdated, persistAfterConnect, waitUntilActive]
  );

  const handleAccountClick = useCallback(
    (account: WeworkSavedAccount) => {
      if (!backendReady || connectingId || disconnecting || switching) {
        return;
      }
      const isActive = channelActive && activeAccountId === account.id;
      if (isActive) {
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

  const handleConfirmSwitch = async () => {
    if (!switchTarget) {
      return;
    }
    const target = switchTarget;
    setSwitching(true);
    try {
      await performDisconnect();
      setSwitchTarget(null);
      await connectAccount(target);
    } catch {
      // keep dialog open on failure
    } finally {
      setSwitching(false);
    }
  };

  const handleNewConnect = async (config: Record<string, string | number | boolean>) => {
    setNewConnecting(true);
    try {
      await actions.connect(config);
      const ok = await waitUntilActive();
      if (ok) {
        const accountConfig = toAccountConfig(config);
        await persistAfterConnect(accountConfig);
        setShowNewForm(false);
        onChannelUpdated?.();
      }
    } catch {
      // keep form open
    } finally {
      setNewConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    if (!window.confirm(t("channels_disconnect_confirm"))) {
      return;
    }
    setDisconnecting(true);
    try {
      await performDisconnect();
      onChannelUpdated?.();
    } catch {
      // noop
    } finally {
      setDisconnecting(false);
    }
  };

  const handleDeleteAccount = (id: string) => {
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
  };

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="shrink-0 border-b border-[hsl(var(--border))] px-6 py-5">
        <div className="flex items-start gap-4">
          <div className="flex size-11 shrink-0 items-center justify-center rounded-2xl bg-[var(--wework-blue-light)]">
            <Building2 className="size-5 text-[var(--wework-blue)]" />
          </div>
          <div>
            <h1 className="text-lg font-bold text-[#1A2B4A]">{t("wework_menu_account")}</h1>
            <p className="mt-0.5 text-sm text-slate-500">{t("wework_accounts_desc")}</p>
          </div>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-4">
        {!accountsLoaded || channelLoading ? (
          <div className="flex items-center justify-center gap-2 py-16 text-sm text-slate-400">
            <Loader2 className="size-4 animate-spin" />
            <span>{t("channels_loading")}</span>
          </div>
        ) : (
          <>
            {channelError ? (
              <div className="mb-4 rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900">
                <p className="font-medium">{t("channels_python_unreachable_title")}</p>
                <p className="mt-2 text-xs opacity-90">{backendErrorMessage}</p>
                <p className="mt-2 text-xs opacity-80">{t("wework_accounts_backend_offline")}</p>
                <button
                  type="button"
                  className="mt-4 cursor-pointer rounded-lg border border-amber-300 px-3 py-1.5 text-xs"
                  onClick={() => onChannelUpdated?.()}
                >
                  {t("channels_retry")}
                </button>
              </div>
            ) : null}

            {channelActive && activeAccount ? (
              <div className="mb-5 flex items-center gap-4 rounded-xl border border-[var(--wework-blue)]/30 bg-white p-4 shadow-sm">
                <AccountAvatar name={activeAccount.label} size="lg" />
                <div className="min-w-0 flex-1">
                  <p className="text-xs text-slate-500">{t("wework_current_account")}</p>
                  <p className="truncate text-lg font-semibold text-[#1A2B4A]">
                    {activeAccount.label}
                  </p>
                  <p className="mt-0.5 flex items-center gap-1.5 text-xs text-emerald-600">
                    <span className="size-1.5 rounded-full bg-emerald-500" />
                    {t("wework_account_connected")}
                  </p>
                </div>
                <button
                  type="button"
                  disabled={disconnecting}
                  className="shrink-0 cursor-pointer rounded-lg border border-red-200 px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 disabled:opacity-50"
                  onClick={() => void handleDisconnect()}
                >
                  {disconnecting ? t("channels_connecting") : t("channels_disconnect")}
                </button>
              </div>
            ) : null}

            {accounts.length === 0 && !showNewForm ? (
              <p className="py-8 text-center text-sm text-slate-400">
                {t("wework_accounts_empty")}
              </p>
            ) : (
              <ul className="space-y-2">
                {accounts.map((account) => {
                  const isActive = channelActive && activeAccountId === account.id;
                  const isConnecting = connectingId === account.id;
                  const path = account.config.wework_exe_path ?? "—";

                  const rowBusy = Boolean(connectingId) || disconnecting || switching;
                  const rowClickable = backendReady && !isActive && !rowBusy;

                  return (
                    <li
                      key={account.id}
                      role={rowClickable ? "button" : undefined}
                      tabIndex={rowClickable ? 0 : -1}
                      className={cn(
                        "rounded-xl border bg-white p-4 shadow-sm transition-colors",
                        isActive
                          ? "border-[var(--wework-blue)]/40 ring-1 ring-[var(--wework-blue)]/20"
                          : "border-[hsl(var(--border))]",
                        rowClickable &&
                          "cursor-pointer hover:border-[var(--wework-blue)]/30 hover:bg-slate-50/80",
                        rowBusy && !isConnecting && "pointer-events-none opacity-60",
                        !backendReady && "opacity-90"
                      )}
                      onClick={() => handleAccountClick(account)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                          e.preventDefault();
                          handleAccountClick(account);
                        }
                      }}
                    >
                      <div className="flex items-center gap-3">
                        <AccountAvatar name={account.label} size="md" />
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <span className="truncate text-base font-semibold text-[#1A2B4A]">
                              {account.label}
                            </span>
                            {isActive ? (
                              <span className="shrink-0 rounded-full bg-emerald-50 px-2 py-0.5 text-[10px] font-medium text-emerald-700">
                                {t("wework_account_connected")}
                              </span>
                            ) : null}
                          </div>
                          <p className="mt-0.5 truncate font-mono text-xs text-slate-500">{path}</p>
                          {account.lastConnectedAt ? (
                            <p className="mt-1 text-[10px] text-slate-400">
                              {t("wework_account_last_connected", {
                                time: new Date(account.lastConnectedAt).toLocaleString()
                              })}
                            </p>
                          ) : null}
                        </div>
                        <div className="relative flex shrink-0 items-center gap-1">
                          {isActive ? (
                            <button
                              type="button"
                              disabled={disconnecting}
                              className="cursor-pointer rounded-lg border border-red-200 px-2.5 py-1 text-xs font-medium text-red-600 hover:bg-red-50 disabled:opacity-50"
                              onClick={(e) => {
                                e.stopPropagation();
                                void handleDisconnect();
                              }}
                            >
                              {disconnecting ? t("channels_connecting") : t("channels_disconnect")}
                            </button>
                          ) : (
                            <button
                              type="button"
                              disabled={rowBusy || !backendReady}
                              className="cursor-pointer rounded-lg bg-[var(--wework-blue)] px-3 py-1 text-xs font-medium text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
                              onClick={(e) => {
                                e.stopPropagation();
                                handleAccountClick(account);
                              }}
                            >
                              {isConnecting ? (
                                <Loader2 className="size-3.5 animate-spin" />
                              ) : (
                                t("wework_account_connect")
                              )}
                            </button>
                          )}
                          <button
                            type="button"
                            className="cursor-pointer rounded-lg p-1 text-slate-400 hover:bg-slate-100"
                            onClick={(e) => {
                              e.stopPropagation();
                              setMenuOpenId((id) => (id === account.id ? null : account.id));
                            }}
                          >
                            <MoreHorizontal className="size-4" />
                          </button>
                          {menuOpenId === account.id ? (
                            <div className="absolute top-8 right-0 z-10 min-w-[7rem] rounded-lg border border-[hsl(var(--border))] bg-white py-1 shadow-lg">
                              <button
                                type="button"
                                className="flex w-full cursor-pointer items-center gap-2 px-3 py-1.5 text-left text-xs text-red-600 hover:bg-red-50"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleDeleteAccount(account.id);
                                }}
                              >
                                <Trash2 className="size-3.5" />
                                {t("wework_account_delete")}
                              </button>
                            </div>
                          ) : null}
                        </div>
                      </div>
                    </li>
                  );
                })}
              </ul>
            )}

            {showNewForm && channel ? (
              <div className="mt-4 rounded-xl border border-[var(--wework-blue)]/25 bg-white p-5 shadow-sm">
                <div className="mb-4 flex items-center gap-2">
                  <Radio className="size-4 text-[var(--wework-blue)]" />
                  <h2 className="font-semibold text-[#1A2B4A]">{t("wework_account_new")}</h2>
                </div>
                <WeworkConnectPanel
                  channel={channel}
                  lang={lang}
                  connecting={newConnecting}
                  onConnect={handleNewConnect}
                  onCancel={() => setShowNewForm(false)}
                />
              </div>
            ) : null}
          </>
        )}
      </div>

      <Dialog
        open={switchTarget !== null}
        onOpenChange={(open) => {
          if (!open && !switching) {
            setSwitchTarget(null);
          }
        }}
      >
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t("wework_account_switch_title")}</DialogTitle>
            <DialogDescription>
              {t("wework_account_switch_message", {
                current: activeAccount?.label ?? "—",
                target: switchTarget?.label ?? "—"
              })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              type="button"
              variant="outline"
              disabled={switching}
              onClick={() => setSwitchTarget(null)}
            >
              {t("wework_account_switch_cancel")}
            </Button>
            <Button
              type="button"
              disabled={switching}
              className="bg-[var(--wework-blue)] text-white hover:opacity-90"
              onClick={() => void handleConfirmSwitch()}
            >
              {switching ? (
                <>
                  <Loader2 className="size-4 animate-spin" />
                  {t("channels_connecting")}
                </>
              ) : (
                t("wework_account_switch_confirm")
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <div className="shrink-0 border-t border-[hsl(var(--border))] bg-white px-6 py-4">
        <button
          type="button"
          disabled={channelLoading || Boolean(channelError) || !channel || showNewForm}
          className="flex w-full cursor-pointer items-center justify-center gap-2 rounded-xl border border-dashed border-[var(--wework-blue)]/50 bg-[var(--wework-blue-light)]/40 py-3 text-sm font-medium text-[var(--wework-blue)] transition-colors hover:bg-[var(--wework-blue-light)] disabled:cursor-not-allowed disabled:opacity-50"
          onClick={() => setShowNewForm(true)}
        >
          <Plus className="size-4" />
          {t("wework_account_new")}
        </button>
      </div>
    </div>
  );
}
