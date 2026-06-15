"use client";

import { Avatar, Badge, Empty, Input, List, Space, Typography } from "@douyinfe/semi-ui-19";
import { IconSearch, IconUserGroup } from "@douyinfe/semi-icons";

import type { WeworkConversationSummary } from "../types/wework-conversation";

const { Text, Title } = Typography;

function formatRelativeTime(ts: number): string {
  const diff = Date.now() - ts;
  if (diff < 60_000) {
    return "刚刚";
  }
  if (diff < 3_600_000) {
    return `${Math.floor(diff / 60_000)} 分钟前`;
  }
  if (diff < 86_400_000) {
    return `${Math.floor(diff / 3_600_000)} 小时前`;
  }
  return `${Math.floor(diff / 86_400_000)} 天前`;
}

export interface ConversationListProps {
  loading: boolean;
  conversations: WeworkConversationSummary[];
  activeConversationId: string | null;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onSelect: (conversationId: string) => void;
}

export function ConversationList({
  loading,
  conversations,
  activeConversationId,
  searchQuery,
  onSearchChange,
  onSelect
}: ConversationListProps) {
  return (
    <List
      className="wework-inbox-list"
      split={false}
      loading={loading}
      dataSource={conversations}
      emptyContent={<Empty description="暂无会话" />}
      header={
        <Space
          vertical
          align="start"
          spacing="tight"
          style={{ width: "100%", padding: "12px 12px 8px" }}
        >
          <Title heading={5} style={{ margin: 0 }}>
            消息
          </Title>
          <Input
            prefix={<IconSearch />}
            showClear
            value={searchQuery}
            onChange={onSearchChange}
            placeholder="搜索群聊或联系人"
            style={{ width: "100%" }}
          />
        </Space>
      }
      renderItem={(item) => {
        const isActive = item.conversationId === activeConversationId;
        const unread = item.unread ?? 0;
        return (
          <List.Item
            onClick={() => onSelect(item.conversationId)}
            style={
              isActive
                ? { backgroundColor: "var(--semi-color-primary-light-default)", cursor: "pointer" }
                : { cursor: "pointer" }
            }
            header={
              unread > 0 ? (
                <Badge count={unread} overflowCount={9} type="primary">
                  <Avatar color="blue" size="medium">
                    {item.kind === "group" ? <IconUserGroup /> : item.title.slice(0, 1)}
                  </Avatar>
                </Badge>
              ) : (
                <Avatar color="blue" size="medium">
                  {item.kind === "group" ? <IconUserGroup /> : item.title.slice(0, 1)}
                </Avatar>
              )
            }
            main={
              <Space vertical align="start" spacing={4} style={{ width: "100%", minWidth: 0 }}>
                <Space style={{ width: "100%", justifyContent: "space-between" }}>
                  <Text strong ellipsis style={{ maxWidth: "10rem" }}>
                    {item.title}
                  </Text>
                  <Text type="tertiary" size="small">
                    {formatRelativeTime(item.lastActive)}
                  </Text>
                </Space>
                <Text type="tertiary" size="small" ellipsis style={{ width: "100%" }}>
                  {item.preview}
                </Text>
              </Space>
            }
          />
        );
      }}
    />
  );
}
