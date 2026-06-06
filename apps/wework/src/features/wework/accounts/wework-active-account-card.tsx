"use client";

import { Button } from "@supportflow/ui/button";

import { AccountAvatar } from "@/features/wework/accounts/account-avatar";
import type { WeworkSavedAccount } from "@/features/wework/accounts/types";

type Translate = (key: string, options?: Record<string, unknown>) => string;

interface WeworkActiveAccountCardProps {
  account: WeworkSavedAccount;
  disconnecting: boolean;
  onDisconnect: () => void;
  onSyncContacts: () => void;
  syncingContacts: boolean;
  t: Translate;
}

export function WeworkActiveAccountCard({
  account,
  disconnecting,
  onDisconnect,
  onSyncContacts,
  syncingContacts,
  t
}: WeworkActiveAccountCardProps) {
  return (
    <div className="border-channel/30 bg-card mb-5 flex items-center gap-4 rounded-xl border p-4 shadow-sm">
      <AccountAvatar name={account.label} size="lg" />
      <div className="min-w-0 flex-1">
        <p className="text-muted-foreground text-xs">{t("wework_current_account")}</p>
        <p className="text-foreground truncate text-lg font-semibold">{account.label}</p>
        <p className="text-success mt-0.5 flex items-center gap-1.5 text-xs">
          <span className="bg-success size-1.5 rounded-full" />
          {t("wework_account_connected")}
        </p>
        <p className="text-muted-foreground mt-1 text-xs">
          {account.contactsSynced ? t("wework_contacts_synced") : t("wework_contacts_not_synced")}
        </p>
      </div>
      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant="outline"
          disabled={disconnecting || syncingContacts}
          className="h-auto px-3 py-1.5 text-xs"
          onClick={onSyncContacts}
        >
          {syncingContacts ? t("channels_connecting") : t("wework_contacts_sync")}
        </Button>
        <Button
          type="button"
          variant="destructive"
          disabled={disconnecting}
          className="h-auto px-3 py-1.5 text-xs"
          onClick={onDisconnect}
        >
          {disconnecting ? t("channels_connecting") : t("channels_disconnect")}
        </Button>
      </div>
    </div>
  );
}
