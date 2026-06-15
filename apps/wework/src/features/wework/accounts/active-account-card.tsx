"use client";

import { Button, Card, Space, Tag, Typography } from "@douyinfe/semi-ui-19";

import { AccountAvatar } from "@/features/wework/accounts/avatar";
import type { WeworkSavedAccount } from "@/features/wework/accounts/types";

const { Text, Title } = Typography;

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
    <Card style={{ marginBottom: 20 }} bodyStyle={{ padding: 16 }}>
      <Space style={{ width: "100%", justifyContent: "space-between" }} align="center">
        <Space align="center">
          <AccountAvatar name={account.label} size="lg" />
          <Space vertical align="start" spacing={4}>
            <Text type="tertiary" size="small">
              当前连接账号
            </Text>
            <Title heading={5} ellipsis style={{ margin: 0, maxWidth: 280 }}>
              {account.label}
            </Title>
            <Tag color="green" size="small">
              已连接
            </Tag>
            <Text type="tertiary" size="small">
              {account.contactsSynced ? "联系人已同步" : "联系人尚未同步"}
            </Text>
          </Space>
        </Space>
        <Space>
          <Button
            theme="light"
            type="tertiary"
            disabled={disconnecting || syncingContacts}
            onClick={onSyncContacts}
          >
            {syncingContacts ? "处理中..." : "同步联系人"}
          </Button>
          <Button type="danger" disabled={disconnecting} onClick={onDisconnect}>
            {disconnecting ? "处理中..." : "断开"}
          </Button>
        </Space>
      </Space>
    </Card>
  );
}
