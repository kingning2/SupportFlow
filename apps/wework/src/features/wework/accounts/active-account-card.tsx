"use client";

import { Button } from "@supportflow/ui/button";

import { AccountAvatar } from "@/features/wework/accounts/avatar";
import type { WeworkSavedAccount } from "@/features/wework/accounts/types";

interface ActiveAccountCardProps {
  account: WeworkSavedAccount;
  disconnecting: boolean;
  onDisconnect: () => void;
  onSyncContacts: () => void;
  syncingContacts: boolean;
}

export function ActiveAccountCard({
  account,
  disconnecting,
  onDisconnect,
  onSyncContacts,
  syncingContacts
}: ActiveAccountCardProps) {
  return (
    <div className="border-channel/30 bg-card mb-5 flex items-center gap-4 rounded-xl border p-4 shadow-sm">
      <AccountAvatar name={account.label} size="lg" />
      <div className="min-w-0 flex-1">
        <p className="text-muted-foreground text-xs">当前连接账号</p>
        <p className="text-foreground truncate text-lg font-semibold">{account.label}</p>
        <p className="text-success mt-0.5 flex items-center gap-1.5 text-xs">
          <span className="bg-success size-1.5 rounded-full" />
          已连接
        </p>
        <p className="text-muted-foreground mt-1 text-xs">
          {account.contactsSynced ? "联系人已同步" : "联系人尚未同步"}
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
          {syncingContacts ? "处理中..." : "同步联系人"}
        </Button>
        <Button
          type="button"
          variant="destructive"
          disabled={disconnecting}
          className="h-auto px-3 py-1.5 text-xs"
          onClick={onDisconnect}
        >
          {disconnecting ? "处理中..." : "断开"}
        </Button>
      </div>
    </div>
  );
}
