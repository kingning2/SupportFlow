"use client";

import { Chat, Empty, Input, Layout, Space, Tag, Typography } from "@douyinfe/semi-ui-19";
import type { Message } from "@douyinfe/semi-ui-19/lib/es/chat/interface";
import { useMemo } from "react";

import type { WeworkConversationSummary, WeworkMessage } from "../types/wework-conversation";

const { Header, Content, Footer } = Layout;
const { Text, Title } = Typography;

function toChatRole(role: WeworkMessage["role"]): Message["role"] {
  if (role === "customer") {
    return "assistant";
  }
  if (role === "system") {
    return "system";
  }
  return "user";
}

function toChatMessages(messages: WeworkMessage[]): Message[] {
  return messages.map((msg) => ({
    id: msg.id,
    role: toChatRole(msg.role),
    content: msg.content,
    createAt: msg.createdAt,
    name: msg.senderName
  }));
}

export interface MessageThreadProps {
  conversation: WeworkConversationSummary | null;
  messages: WeworkMessage[];
}

export function MessageThread({ conversation, messages }: MessageThreadProps) {
  const chats = useMemo(() => toChatMessages(messages), [messages]);

  if (!conversation) {
    return (
      <Content className="wework-inbox-thread" style={{ display: "flex", minHeight: 0, flex: 1 }}>
        <Empty
          style={{ margin: "auto" }}
          title="选择一个会话开始查看"
          description="从左侧列表选择群聊或联系人"
        />
      </Content>
    );
  }

  return (
    <Layout className="wework-inbox-thread" style={{ height: "100%", minHeight: 0, flex: 1 }}>
      <Header
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "12px 16px",
          height: "auto",
          lineHeight: "inherit"
        }}
      >
        <Space vertical align="start" spacing={2} style={{ minWidth: 0 }}>
          <Title heading={6} ellipsis style={{ margin: 0, maxWidth: "100%" }}>
            {conversation.title}
          </Title>
          <Text type="tertiary" size="small" ellipsis style={{ maxWidth: "100%" }}>
            {conversation.conversationId}
          </Text>
        </Space>
        <Tag color="blue" size="small">
          {conversation.kind === "group" ? "群聊" : "单聊"}
        </Tag>
      </Header>

      <Content style={{ flex: 1, minHeight: 0, display: "flex", flexDirection: "column" }}>
        {messages.length === 0 ? (
          <Empty style={{ margin: "auto" }} description="暂无消息" />
        ) : (
          <Chat
            mode="bubble"
            align="leftRight"
            chats={chats}
            canSend={false}
            showClearContext={false}
            style={{ flex: 1, minHeight: 0, border: "none" }}
            chatBoxRenderConfig={{
              renderChatBoxAction: () => null,
              renderChatBoxTitle: ({ message, defaultTitle }) =>
                message?.name ? (
                  <Text type="tertiary" size="small">
                    {message.name}
                  </Text>
                ) : (
                  defaultTitle
                )
            }}
            renderInputArea={() => null}
          />
        )}
      </Content>

      <Footer
        style={{
          padding: "12px 16px",
          height: "auto",
          lineHeight: "inherit",
          borderTop: "1px solid var(--semi-color-border)"
        }}
      >
        <Space vertical align="center" spacing="tight" style={{ width: "100%" }}>
          <Input
            disabled
            placeholder="人工回复（即将支持）"
            style={{ width: "100%", maxWidth: 768 }}
          />
          <Text type="tertiary" size="small">
            消息已从渠道实时同步
          </Text>
        </Space>
      </Footer>
    </Layout>
  );
}
