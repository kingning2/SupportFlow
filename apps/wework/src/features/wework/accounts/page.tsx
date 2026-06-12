"use client";

import { Building2, Loader2, Plus, Radio } from "lucide-react";

import { Button } from "@supportflow/ui/button";
import { WeworkConnectPanel } from "@supportflow/ui/channel/wework-connect-panel";

import { ActiveAccountCard } from "@/features/wework/accounts/active-account-card";
import { AccountList } from "@/features/wework/accounts/list";
import { AccountSwitchDialog } from "@/features/wework/accounts/switch-dialog";
import type { PageActions } from "@/features/wework/accounts/page-types";
import type { WeworkConnectionStatus } from "@/features/wework/types/wework-conversation";

import type { PageHandlers, PageState } from "./page-state";
import { usePageState } from "./page-state";
import type { ChannelCatalogEntry } from "@supportflow/shared";

export interface PageProps {
  lang: string;
  actions: PageActions;
  channel: ChannelCatalogEntry | null;
  channelLoading: boolean;
  channelError: string | null;
  connectionStatus: WeworkConnectionStatus;
  onChannelUpdated?: () => void;
}

function PageHeader({ title, description }: { title: string; description: string }) {
  return (
    <div className="shrink-0 border-b border-[hsl(var(--border))] px-6 py-5">
      <div className="flex items-start gap-4">
        <div className="bg-channel-muted flex size-11 shrink-0 items-center justify-center rounded-2xl">
          <Building2 className="text-channel size-5" />
        </div>
        <div>
          <h1 className="text-foreground text-lg font-bold">{title}</h1>
          <p className="text-muted-foreground mt-0.5 text-sm">{description}</p>
        </div>
      </div>
    </div>
  );
}

function PageLoading({ text }: { text: string }) {
  return (
    <div className="text-muted-foreground flex items-center justify-center gap-2 py-16 text-sm">
      <Loader2 className="size-4 animate-spin" />
      <span>{text}</span>
    </div>
  );
}

function PageError({
  backendErrorMessage,
  offlineText,
  retryLabel,
  title,
  onRetry
}: {
  backendErrorMessage: string | null;
  offlineText: string;
  retryLabel: string;
  title: string;
  onRetry: () => void;
}) {
  if (!backendErrorMessage) {
    return null;
  }

  return (
    <div className="bg-warning/10 text-warning-foreground border-warning/30 mb-4 rounded-xl border p-4 text-sm">
      <p className="font-medium">{title}</p>
      <p className="mt-2 text-xs opacity-90">{backendErrorMessage}</p>
      <p className="mt-2 text-xs opacity-80">{offlineText}</p>
      <Button
        type="button"
        variant="outline"
        className="border-warning/40 mt-4 text-xs"
        onClick={onRetry}
      >
        {retryLabel}
      </Button>
    </div>
  );
}

function AccountsSection({
  channel,
  handlers,
  lang,
  state
}: {
  channel: ChannelCatalogEntry | null;
  handlers: PageHandlers;
  lang: string;
  state: PageState;
}) {
  return (
    <>
      {state.channelActive && state.activeAccount ? (
        <ActiveAccountCard
          account={state.activeAccount}
          disconnecting={state.disconnecting}
          onDisconnect={() => void handlers.handleDisconnect()}
          onSyncContacts={() => void handlers.handleSyncContacts()}
          syncingContacts={state.syncingContacts}
        />
      ) : null}

      {state.accounts.length === 0 && !state.showNewForm ? (
        <p className="text-muted-foreground py-8 text-center text-sm">
          暂无已保存账号，点击下方“新建连接”添加。
        </p>
      ) : (
        <AccountList
          accounts={state.accounts}
          activeAccountId={state.activeAccountId}
          backendReady={state.backendReady}
          channelActive={state.channelActive}
          connectingId={state.connectingId}
          disconnecting={state.disconnecting}
          menuOpenId={state.menuOpenId}
          onAccountClick={handlers.handleAccountClick}
          onDeleteAccount={handlers.handleDeleteAccount}
          onDisconnect={() => void handlers.handleDisconnect()}
          onMenuToggle={(id: string) =>
            handlers.setMenuOpenId((current) => (current === id ? null : id))
          }
          switching={state.switching}
        />
      )}

      {state.showNewForm && channel ? (
        <div className="border-channel/25 bg-card mt-4 rounded-xl border p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <Radio className="text-channel size-4" />
            <h2 className="text-foreground font-semibold">新建连接</h2>
          </div>
          <WeworkConnectPanel
            channel={channel}
            lang={lang}
            connecting={state.newConnecting}
            onCancel={() => handlers.setShowNewForm(false)}
            onConnect={handlers.handleNewConnect}
          />
        </div>
      ) : null}
    </>
  );
}

export function Page({
  lang,
  actions,
  channel,
  channelLoading,
  channelError,
  connectionStatus,
  onChannelUpdated
}: PageProps) {
  const { handlers, state } = usePageState({
    actions,
    channel,
    channelError,
    channelLoading,
    connectionStatus,
    onChannelUpdated
  });

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <PageHeader
        title="账号与通道"
        description="保存常用企业微信配置，需要时手动连接；连接成功后会自动加入列表。"
      />

      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-4">
        {!state.accountsLoaded || channelLoading ? (
          <PageLoading text="加载通道配置中..." />
        ) : (
          <>
            <PageError
              backendErrorMessage={state.backendErrorMessage}
              offlineText="当前无法连接通道服务，下方展示的是本地保存的账号；连接与切换请等服务恢复后再试。"
              retryLabel="重试"
              title="通道 sidecar 未就绪"
              onRetry={handlers.onRetry}
            />

            <AccountsSection channel={channel} handlers={handlers} lang={lang} state={state} />
          </>
        )}
      </div>

      <AccountSwitchDialog
        activeAccountLabel={state.activeAccount?.label}
        onConfirm={() => void handlers.handleConfirmSwitch()}
        onOpenChange={handlers.handleSwitchOpenChange}
        switching={state.switching}
        switchTarget={state.switchTarget}
      />

      <div className="shrink-0 border-t border-[hsl(var(--border))] bg-white px-6 py-4">
        <Button
          type="button"
          variant="outline"
          disabled={channelLoading || Boolean(channelError) || !channel || state.showNewForm}
          className="border-channel/50 bg-channel-muted/40 text-channel hover:bg-channel-muted flex w-full items-center justify-center gap-2 border-dashed py-3 text-sm font-medium"
          onClick={() => handlers.setShowNewForm(true)}
        >
          <Plus className="size-4" />
          新建连接
        </Button>
      </div>
    </div>
  );
}
