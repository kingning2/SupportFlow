"use client";

import { useState } from "react";
import { Descriptions, IconButton, Layout, Space, Tag, Typography } from "@douyinfe/semi-ui-19";
import { IconChevronLeft, IconChevronRight } from "@douyinfe/semi-icons";

import type { WeworkConversationSummary } from "../types/wework-conversation";

const { Sider, Header, Content } = Layout;
const { Text } = Typography;

const DETAIL_WIDTH = 256;
const COLLAPSED_WIDTH = 40;

export interface ConversationDetailProps {
  conversation: WeworkConversationSummary | null;
}

export function ConversationDetail({ conversation }: ConversationDetailProps) {
  const [collapsed, setCollapsed] = useState(true);

  if (collapsed) {
    return (
      <Sider
        className="wework-inbox-detail wework-inbox-detail--collapsed"
        style={{
          width: COLLAPSED_WIDTH,
          flexShrink: 0,
          display: "flex",
          alignItems: "stretch",
          borderLeft: "1px solid var(--semi-color-border)"
        }}
      >
        <IconButton
          icon={<IconChevronLeft />}
          type="tertiary"
          aria-label="展开详情"
          onClick={() => setCollapsed(false)}
          style={{ flex: 1, height: "100%", borderRadius: 0 }}
        />
      </Sider>
    );
  }

  return (
    <Sider className="wework-inbox-detail" style={{ width: DETAIL_WIDTH, flexShrink: 0 }}>
      <Layout style={{ height: "100%" }}>
        <Header
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "12px",
            height: "auto",
            lineHeight: "inherit"
          }}
        >
          <Text strong>会话详情</Text>
          <IconButton
            icon={<IconChevronRight />}
            type="tertiary"
            size="small"
            aria-label="收起详情"
            onClick={() => setCollapsed(true)}
          />
        </Header>
        <Content style={{ flex: 1, minHeight: 0, overflow: "auto", padding: "12px 16px" }}>
          {!conversation ? (
            <Text type="tertiary">选择会话后显示详情</Text>
          ) : (
            <Space vertical align="start" spacing="medium" style={{ width: "100%" }}>
              <Descriptions
                align="left"
                row
                size="small"
                data={[
                  {
                    key: "Agent Session",
                    value: (
                      <Text code copyable>
                        {conversation.sessionId}
                      </Text>
                    )
                  },
                  {
                    key: "Conversation ID",
                    value: (
                      <Text code copyable>
                        {conversation.conversationId}
                      </Text>
                    )
                  },
                  {
                    key: "群 AI",
                    value: (
                      <Tag color="blue" size="small">
                        已启用
                      </Tag>
                    )
                  }
                ]}
              />
              <Text type="tertiary" size="small">
                当前为演示数据；一群一会话，session 映射为 wework:{"{conversationId}"}。
              </Text>
            </Space>
          )}
        </Content>
      </Layout>
    </Sider>
  );
}
