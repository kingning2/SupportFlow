"use client";

import { useWeworkInbox } from "../hooks/use-wework-inbox";
import type { WeworkConnectionStatus } from "../types/wework-conversation";
import { ConversationDetail } from "./conversation-detail";
import { ConversationList } from "./conversation-list";
import { MessageThread } from "./message-thread";

export interface InboxProps {
  connectionStatus: WeworkConnectionStatus;
}

export function Inbox({ connectionStatus }: InboxProps) {
  const inbox = useWeworkInbox({ connectionStatus });

  return (
    <div className="inbox-shell flex min-h-0 min-w-0 flex-1 gap-3 overflow-hidden p-3">
      <ConversationList
        loading={inbox.loading}
        conversations={inbox.conversations}
        activeConversationId={inbox.activeConversationId}
        searchQuery={inbox.searchQuery}
        onSearchChange={inbox.setSearchQuery}
        onSelect={inbox.setActiveConversationId}
      />
      <MessageThread conversation={inbox.activeConversation} messages={inbox.activeMessages} />
      <ConversationDetail conversation={inbox.activeConversation} />
    </div>
  );
}
