/** 企微会话类型：单聊 / 群聊 */
export type WeworkConversationKind = "group" | "direct";

/** 消息发送方角色（展示用） */
export type WeworkMessageRole = "customer" | "operator" | "assistant" | "system";

export interface WeworkConversationSummary {
  conversationId: string;
  /** CowAgent session_id，稳定映射 wework:{conversationId} */
  sessionId: string;
  title: string;
  kind: WeworkConversationKind;
  lastActive: number;
  preview: string;
  unread?: number;
}

export interface WeworkMessage {
  id: string;
  conversationId: string;
  role: WeworkMessageRole;
  senderName?: string;
  content: string;
  createdAt: number;
}

export type WeworkConnectionStatus = "disconnected" | "connecting" | "ready";
