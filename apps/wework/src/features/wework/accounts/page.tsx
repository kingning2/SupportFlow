"use client";

import {
  Avatar,
  Banner,
  Button,
  Card,
  Empty,
  Layout,
  Space,
  Spin,
  Typography
} from "@douyinfe/semi-ui-19";
import { IconApartment, IconLink, IconPlus } from "@douyinfe/semi-icons";

import { WeworkConnectPanel } from "@supportflow/ui/channel/wework-connect-panel";

import { ActiveAccountCard } from "@/features/wework/accounts/active-account-card";
import { AccountList } from "@/features/wework/accounts/list";
import { AccountSwitchDialog } from "@/features/wework/accounts/switch-dialog";
import type { PageActions } from "@/features/wework/accounts/page-types";
import type { WeworkConnectionStatus } from "@/features/wework/types/wework-conversation";

import type { PageHandlers, PageState } from "./page-state";
import { usePageState } from "./page-state";
import type { ChannelCatalogEntry } from "@supportflow/shared";

const { Header, Content, Footer } = Layout;
const { Title, Text } = Typography;

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
    <Header
      className="wework-panel-header"
      style={{ height: "auto", lineHeight: "inherit", padding: "20px 24px" }}
    >
      <Space align="start">
        <Avatar
          size="large"
          style={{
            background: "linear-gradient(145deg, #3370ff 0%, #245bdb 100%)",
            color: "#fff"
          }}
        >
          <IconApartment />
        </Avatar>
        <Space vertical align="start" spacing={4}>
          <Title heading={4} style={{ margin: 0 }}>
            {title}
          </Title>
          <Text type="tertiary">{description}</Text>
        </Space>
      </Space>
    </Header>
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
    <Banner
      fullMode={false}
      bordered
      type="warning"
      closeIcon={null}
      title={title}
      description={
        <Space vertical align="start" spacing="tight">
          <Text size="small">{backendErrorMessage}</Text>
          <Text type="tertiary" size="small">
            {offlineText}
          </Text>
          <Button theme="light" type="warning" size="small" onClick={onRetry}>
            {retryLabel}
          </Button>
        </Space>
      }
      style={{ marginBottom: 16 }}
    />
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
        <Empty description="暂无已保存账号，点击下方「新建连接」添加。" />
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
        <Card
          style={{ marginTop: 16 }}
          title={
            <Space>
              <IconLink />
              <span>新建连接</span>
            </Space>
          }
        >
          <WeworkConnectPanel
            channel={channel}
            lang={lang}
            connecting={state.newConnecting}
            onCancel={() => handlers.setShowNewForm(false)}
            onConnect={handlers.handleNewConnect}
          />
        </Card>
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
    <Layout style={{ height: "100%", minHeight: 0 }}>
      <PageHeader
        title="账号与通道"
        description="保存常用企业微信配置，需要时手动连接；连接成功后会自动加入列表。"
      />

      <Content style={{ flex: 1, minHeight: 0, overflow: "auto", padding: "16px 24px" }}>
        {!state.accountsLoaded || channelLoading ? (
          <Spin tip="加载通道配置中..." style={{ display: "block", margin: "64px auto" }} />
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
      </Content>

      <AccountSwitchDialog
        activeAccountLabel={state.activeAccount?.label}
        onConfirm={() => void handlers.handleConfirmSwitch()}
        onOpenChange={handlers.handleSwitchOpenChange}
        switching={state.switching}
        switchTarget={state.switchTarget}
      />

      <Footer
        className="wework-panel-header"
        style={{ height: "auto", lineHeight: "inherit", padding: "16px 24px" }}
      >
        <Button
          block
          theme="light"
          type="primary"
          icon={<IconPlus />}
          disabled={channelLoading || Boolean(channelError) || !channel || state.showNewForm}
          onClick={() => handlers.setShowNewForm(true)}
        >
          新建连接
        </Button>
      </Footer>
    </Layout>
  );
}
