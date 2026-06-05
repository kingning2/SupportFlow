import type { WeworkConversationSummary, WeworkMessage } from "../types/wework-conversation";

export function weworkSessionId(conversationId: string): string {
  return `wework:${conversationId}`;
}

const now = Date.now();

export const MOCK_WEWORK_CONVERSATIONS: WeworkConversationSummary[] = [
  {
    conversationId: "R:mock-vip-001",
    sessionId: weworkSessionId("R:mock-vip-001"),
    title: "VIP 客户服务群",
    kind: "group",
    lastActive: now - 120_000,
    preview: "请问发票什么时候能开？",
    unread: 2
  },
  {
    conversationId: "R:mock-product-002",
    sessionId: weworkSessionId("R:mock-product-002"),
    title: "产品咨询 · 华东",
    kind: "group",
    lastActive: now - 3_600_000,
    preview: "智能客服：已为您查询到订单状态…",
    unread: 0
  },
  {
    conversationId: "S:mock-direct-003",
    sessionId: weworkSessionId("S:mock-direct-003"),
    title: "张先生",
    kind: "direct",
    lastActive: now - 86_400_000,
    preview: "好的，谢谢",
    unread: 0
  }
];

export const MOCK_WEWORK_MESSAGES: Record<string, WeworkMessage[]> = {
  "R:mock-vip-001": [
    {
      id: "m1",
      conversationId: "R:mock-vip-001",
      role: "customer",
      senderName: "李女士",
      content: "你好，我们想加购企业版席位。",
      createdAt: now - 600_000
    },
    {
      id: "m2",
      conversationId: "R:mock-vip-001",
      role: "assistant",
      content: "您好，已记录需求。请问预计新增多少席位？",
      createdAt: now - 540_000
    },
    {
      id: "m3",
      conversationId: "R:mock-vip-001",
      role: "customer",
      senderName: "李女士",
      content: "请问发票什么时候能开？",
      createdAt: now - 120_000
    }
  ],
  "R:mock-product-002": [
    {
      id: "m4",
      conversationId: "R:mock-product-002",
      role: "customer",
      senderName: "王工",
      content: "订单 #8821 物流到哪了？",
      createdAt: now - 4_000_000
    },
    {
      id: "m5",
      conversationId: "R:mock-product-002",
      role: "assistant",
      content: "智能客服：已为您查询到订单状态，预计明日送达。",
      createdAt: now - 3_600_000
    }
  ],
  "S:mock-direct-003": [
    {
      id: "m6",
      conversationId: "S:mock-direct-003",
      role: "operator",
      content: "您好，售后问题已处理完毕。",
      createdAt: now - 90_000_000
    },
    {
      id: "m7",
      conversationId: "S:mock-direct-003",
      role: "customer",
      senderName: "张先生",
      content: "好的，谢谢",
      createdAt: now - 86_400_000
    }
  ]
};
