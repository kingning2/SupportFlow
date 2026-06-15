"use client";

import { Layout } from "@douyinfe/semi-ui-19";

import { useWeworkInbox } from "../hooks/use-wework-inbox";
import type { WeworkConnectionStatus } from "../types/wework-conversation";
import { ConversationDetail } from "./conversation-detail";
import { ConversationList } from "./conversation-list";
import { MessageThread } from "./message-thread";

const { Sider, Content } = Layout;

export interface InboxProps {
  connectionStatus: WeworkConnectionStatus;
}

export function Inbox({ connectionStatus }: InboxProps) {
  const inbox = useWeworkInbox({ connectionStatus });

  return (
    <Layout
      className="wework-inbox-shell"
      style={{ flex: 1, minHeight: 0, minWidth: 0, overflow: "hidden" }}
    >
      <Sider className="wework-inbox-list-sider">
        <ConversationList
          loading={inbox.loading}
          conversations={inbox.conversations}
          activeConversationId={inbox.activeConversationId}
          searchQuery={inbox.searchQuery}
          onSearchChange={inbox.setSearchQuery}
          onSelect={inbox.setActiveConversationId}
        />
      </Sider>
      <Content style={{ minWidth: 0, minHeight: 0, display: "flex", flex: 1 }}>
        <MessageThread conversation={inbox.activeConversation} messages={inbox.activeMessages} />
      </Content>
      <ConversationDetail conversation={inbox.activeConversation} />
    </Layout>
  );
}
