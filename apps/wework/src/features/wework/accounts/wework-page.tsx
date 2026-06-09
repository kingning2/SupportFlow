"use client";

import { Building2, Loader2, Plus, Radio } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@supportflow/ui/button";
import { WeworkConnectPanel } from "@supportflow/ui/channel/wework-connect-panel";

import { WeworkActiveAccountCard } from "@/features/wework/accounts/wework-active-account-card";
import { WeworkAccountList } from "@/features/wework/accounts/wework-account-list";
import { WeworkAccountSwitchDialog } from "@/features/wework/accounts/wework-account-switch-dialog";
import type { WeworkPageActions } from "@/features/wework/accounts/wework-page-types";
import type { WeworkConnectionStatus } from "@/features/wework/types/wework-conversation";

import type { WeworkPageHandlers, WeworkPageState } from "./wework-page-state";
import { useWeworkPageState } from "./wework-page-state";
import type { ChannelCatalogEntry } from "@supportflow/shared";

type Translate = (key: string, options?: Record<string, unknown>) => string;

export interface WeworkPageProps {
  lang: string;
  actions: WeworkPageActions;
  channel: ChannelCatalogEntry | null;
  channelLoading: boolean;
  channelError: string | null;
  connectionStatus: WeworkConnectionStatus;
  onChannelUpdated?: () => void;
}

function WeworkPageHeader({ title, description }: { title: string; description: string }) {
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

function WeworkPageLoading({ text }: { text: string }) {
  return (
    <div className="text-muted-foreground flex items-center justify-center gap-2 py-16 text-sm">
      <Loader2 className="size-4 animate-spin" />
      <span>{text}</span>
    </div>
  );
}

function WeworkPageError({
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

function WeworkPageAccountsSection({
  channel,
  handlers,
  lang,
  state,
  t
}: {
  channel: ChannelCatalogEntry | null;
  handlers: WeworkPageHandlers;
  lang: string;
  state: WeworkPageState;
  t: Translate;
}) {
  return (
    <>
      {state.channelActive && state.activeAccount ? (
        <WeworkActiveAccountCard
          account={state.activeAccount}
          disconnecting={state.disconnecting}
          onDisconnect={() => void handlers.handleDisconnect()}
          onSyncContacts={() => void handlers.handleSyncContacts()}
          syncingContacts={state.syncingContacts}
          t={t}
        />
      ) : null}

      {state.accounts.length === 0 && !state.showNewForm ? (
        <p className="text-muted-foreground py-8 text-center text-sm">
          {"暂无已保存账号，点击下方「新建连接」添加。"}
        </p>
      ) : (
        <WeworkAccountList
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
          onMenuToggle={(id) => handlers.setMenuOpenId((current) => (current === id ? null : id))}
          switching={state.switching}
          t={t}
        />
      )}

      {state.showNewForm && channel ? (
        <div className="border-channel/25 bg-card mt-4 rounded-xl border p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <Radio className="text-channel size-4" />
            <h2 className="text-foreground font-semibold">{"新建连接"}</h2>
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
  const { handlers, state } = useWeworkPageState({
    actions,
    channel,
    channelError,
    channelLoading,
    connectionStatus,
    onChannelUpdated,
    t
  });

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <WeworkPageHeader
        title={"账号与通道"}
        description={"保存常用企微配置，需要时手动连接；连接成功后会自动加入列表。"}
      />

      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-4">
        {!state.accountsLoaded || channelLoading ? (
          <WeworkPageLoading text={"加载通道配置…"} />
        ) : (
          <>
            <WeworkPageError
              backendErrorMessage={state.backendErrorMessage}
              offlineText={
                "无法连接通道服务，以下为本地已保存的账号；连接与切换需等服务就绪后再试。"
              }
              retryLabel={"重试"}
              title={"通道 sidecar 未就绪"}
              onRetry={handlers.onRetry}
            />

            <WeworkPageAccountsSection
              channel={channel}
              handlers={handlers}
              lang={lang}
              state={state}
              t={t}
            />
          </>
        )}
      </div>

      <WeworkAccountSwitchDialog
        activeAccountLabel={state.activeAccount?.label}
        onConfirm={() => void handlers.handleConfirmSwitch()}
        onOpenChange={handlers.handleSwitchOpenChange}
        switching={state.switching}
        switchTarget={state.switchTarget}
        t={t}
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
          {"新建连接"}
        </Button>
      </div>
    </div>
  );
}
