"use client";

import type { KeyboardEvent } from "react";
import { Loader2, MoreHorizontal, Trash2 } from "lucide-react";

import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";

import { AccountAvatar } from "@/features/wework/accounts/account-avatar";
import type { WeworkSavedAccount } from "@/features/wework/accounts/types";

type Translate = (key: string, options?: Record<string, unknown>) => string;

interface WeworkAccountListProps {
  accounts: WeworkSavedAccount[];
  activeAccountId: string | null;
  backendReady: boolean;
  channelActive: boolean;
  connectingId: string | null;
  disconnecting: boolean;
  menuOpenId: string | null;
  onAccountClick: (account: WeworkSavedAccount) => void;
  onDeleteAccount: (id: string) => void;
  onDisconnect: () => void;
  onMenuToggle: (id: string) => void;
  switching: boolean;
  t: Translate;
}

interface WeworkAccountListItemProps extends WeworkAccountListProps {
  account: WeworkSavedAccount;
}

function canTriggerRow(event: KeyboardEvent<HTMLLIElement>) {
  return event.key === "Enter" || event.key === " ";
}

function renderConnectionAction(params: {
  account: WeworkSavedAccount;
  backendReady: boolean;
  disconnecting: boolean;
  isActive: boolean;
  isConnecting: boolean;
  onAccountClick: (account: WeworkSavedAccount) => void;
  onDisconnect: () => void;
  rowBusy: boolean;
  t: Translate;
}) {
  const {
    account,
    backendReady,
    disconnecting,
    isActive,
    isConnecting,
    onAccountClick,
    onDisconnect,
    rowBusy,
    t
  } = params;

  if (isActive) {
    return (
      <Button
        type="button"
        variant="destructive"
        disabled={disconnecting}
        className="h-auto px-2.5 py-1 text-xs"
        onClick={(event) => {
          event.stopPropagation();
          onDisconnect();
        }}
      >
        {disconnecting ? t("channels_connecting") : t("channels_disconnect")}
      </Button>
    );
  }

  return (
    <Button
      type="button"
      disabled={rowBusy || !backendReady}
      className="bg-channel text-channel-foreground hover:bg-channel/90 h-auto px-3 py-1 text-xs"
      onClick={(event) => {
        event.stopPropagation();
        onAccountClick(account);
      }}
    >
      {isConnecting ? <Loader2 className="size-3.5 animate-spin" /> : t("wework_account_connect")}
    </Button>
  );
}

function WeworkAccountListItem({
  account,
  activeAccountId,
  backendReady,
  channelActive,
  connectingId,
  disconnecting,
  menuOpenId,
  onAccountClick,
  onDeleteAccount,
  onDisconnect,
  onMenuToggle,
  switching,
  t
}: WeworkAccountListItemProps) {
  const isActive = channelActive && activeAccountId === account.id;
  const isConnecting = connectingId === account.id;
  const path = account.config.wework_exe_path ?? "N/A";
  const rowBusy = Boolean(connectingId) || disconnecting || switching;
  const rowClickable = backendReady && !isActive && !rowBusy;

  return (
    <li
      role={rowClickable ? "button" : undefined}
      tabIndex={rowClickable ? 0 : -1}
      className={cn(
        "rounded-xl border bg-white p-4 shadow-sm transition-colors",
        "bg-card",
        isActive ? "border-channel/40 ring-channel/20 ring-1" : "border-[hsl(var(--border))]",
        rowClickable && "hover:border-channel/30 hover:bg-accent/40 cursor-pointer",
        rowBusy && !isConnecting && "pointer-events-none opacity-60",
        !backendReady && "opacity-90"
      )}
      onClick={() => onAccountClick(account)}
      onKeyDown={(event) => {
        if (!canTriggerRow(event)) {
          return;
        }
        event.preventDefault();
        onAccountClick(account);
      }}
    >
      <div className="flex items-center gap-3">
        <AccountAvatar name={account.label} size="md" />
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-foreground truncate text-base font-semibold">
              {account.label}
            </span>
            {isActive ? (
              <span className="bg-success/10 text-success shrink-0 rounded-full px-2 py-0.5 text-[10px] font-medium">
                {t("wework_account_connected")}
              </span>
            ) : null}
          </div>
          <p className="text-muted-foreground mt-0.5 truncate font-mono text-xs">{path}</p>
          {account.lastConnectedAt ? (
            <p className="text-muted-foreground mt-1 text-[10px]">
              {t("wework_account_last_connected", {
                time: new Date(account.lastConnectedAt).toLocaleString()
              })}
            </p>
          ) : null}
          <p className="text-muted-foreground mt-1 text-[10px]">
            {account.contactsSynced ? t("wework_contacts_synced") : t("wework_contacts_not_synced")}
          </p>
        </div>
        <div className="relative flex shrink-0 items-center gap-1">
          {renderConnectionAction({
            account,
            backendReady,
            disconnecting,
            isActive,
            isConnecting,
            onAccountClick,
            onDisconnect,
            rowBusy,
            t
          })}
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="text-muted-foreground"
            onClick={(event) => {
              event.stopPropagation();
              onMenuToggle(account.id);
            }}
          >
            <MoreHorizontal className="size-4" />
          </Button>
          {menuOpenId === account.id ? (
            <div className="bg-card border-border absolute top-8 right-0 z-10 min-w-[7rem] rounded-lg border py-1 shadow-lg">
              <Button
                type="button"
                variant="ghost"
                className="text-destructive hover:bg-destructive/10 flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs"
                onClick={(event) => {
                  event.stopPropagation();
                  onDeleteAccount(account.id);
                }}
              >
                <Trash2 className="size-3.5" />
                {t("wework_account_delete")}
              </Button>
            </div>
          ) : null}
        </div>
      </div>
    </li>
  );
}

export function WeworkAccountList(props: WeworkAccountListProps) {
  return (
    <ul className="space-y-2">
      {props.accounts.map((account) => (
        <WeworkAccountListItem key={account.id} {...props} account={account} />
      ))}
    </ul>
  );
}
