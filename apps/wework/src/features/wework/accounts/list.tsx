"use client";

import { Button, Dropdown, IconButton, List, Space, Tag, Typography } from "@douyinfe/semi-ui-19";
import { IconDelete, IconMoreStroked } from "@douyinfe/semi-icons";

import { AccountAvatar } from "@/features/wework/accounts/avatar";
import type { WeworkSavedAccount } from "@/features/wework/accounts/types";

const { Text } = Typography;

interface AccountListProps {
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
}) {
  const {
    account,
    backendReady,
    disconnecting,
    isActive,
    isConnecting,
    onAccountClick,
    onDisconnect,
    rowBusy
  } = params;

  if (isActive) {
    return (
      <Button
        type="danger"
        size="small"
        disabled={disconnecting}
        onClick={(event) => {
          event.stopPropagation();
          onDisconnect();
        }}
      >
        {disconnecting ? "处理中" : "断开"}
      </Button>
    );
  }

  return (
    <Button
      type="primary"
      size="small"
      disabled={rowBusy || !backendReady}
      loading={isConnecting}
      onClick={(event) => {
        event.stopPropagation();
        onAccountClick(account);
      }}
    >
      连接
    </Button>
  );
}

export function AccountList({
  accounts,
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
  switching
}: AccountListProps) {
  return (
    <List
      split
      dataSource={accounts}
      renderItem={(account) => {
        const isActive = channelActive && activeAccountId === account.id;
        const isConnecting = connectingId === account.id;
        const path = account.config.wework_exe_path ?? "N/A";
        const rowBusy = Boolean(connectingId) || disconnecting || switching;
        const rowClickable = backendReady && !isActive && !rowBusy;

        return (
          <List.Item
            style={{
              cursor: rowClickable ? "pointer" : "default",
              opacity: rowBusy && !isConnecting ? 0.6 : 1,
              backgroundColor: isActive ? "var(--semi-color-primary-light-default)" : undefined
            }}
            onClick={() => {
              if (rowClickable) {
                onAccountClick(account);
              }
            }}
            header={<AccountAvatar name={account.label} size="md" />}
            main={
              <Space vertical align="start" spacing={4} style={{ minWidth: 0, flex: 1 }}>
                <Space align="center" spacing="tight">
                  <Text strong ellipsis style={{ maxWidth: 200 }}>
                    {account.label}
                  </Text>
                  {isActive ? (
                    <Tag color="green" size="small">
                      已连接
                    </Tag>
                  ) : null}
                </Space>
                <Text type="tertiary" size="small" code ellipsis style={{ maxWidth: "100%" }}>
                  {path}
                </Text>
                {account.lastConnectedAt ? (
                  <Text type="tertiary" size="small">
                    {`最近连接：${new Date(account.lastConnectedAt).toLocaleString()}`}
                  </Text>
                ) : null}
                <Text type="tertiary" size="small">
                  {account.contactsSynced ? "联系人已同步" : "联系人未同步"}
                </Text>
              </Space>
            }
            extra={
              <Space>
                {renderConnectionAction({
                  account,
                  backendReady,
                  disconnecting,
                  isActive,
                  isConnecting,
                  onAccountClick,
                  onDisconnect,
                  rowBusy
                })}
                <Dropdown
                  trigger="click"
                  position="bottomRight"
                  visible={menuOpenId === account.id}
                  onVisibleChange={(visible) => {
                    if (visible) {
                      onMenuToggle(account.id);
                    } else if (menuOpenId === account.id) {
                      onMenuToggle(account.id);
                    }
                  }}
                  render={
                    <Dropdown.Menu>
                      <Dropdown.Item
                        icon={<IconDelete />}
                        type="danger"
                        onClick={(e) => {
                          e.stopPropagation();
                          onDeleteAccount(account.id);
                        }}
                      >
                        删除
                      </Dropdown.Item>
                    </Dropdown.Menu>
                  }
                >
                  <IconButton
                    icon={<IconMoreStroked />}
                    type="tertiary"
                    theme="borderless"
                    onClick={(e) => e.stopPropagation()}
                  />
                </Dropdown>
              </Space>
            }
          />
        );
      }}
    />
  );
}
