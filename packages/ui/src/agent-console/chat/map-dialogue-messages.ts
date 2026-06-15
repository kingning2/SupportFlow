import type {
  ContentItem,
  Message
} from "@douyinfe/semi-foundation/lib/es/aiChatDialogue/foundation";

import type { AssistantChatMessage, ChatMessage } from "../types/agent-chat";

function toolStepStatus(status: AssistantChatMessage["toolSteps"][number]["status"]) {
  if (status === "running") {
    return "in_progress";
  }
  if (status === "error") {
    return "failed";
  }
  return "completed";
}

function assistantDialogueStatus(message: AssistantChatMessage): string {
  if (message.cancelled) {
    return "cancelled";
  }
  if (message.streaming) {
    const hasBody =
      Boolean(message.content) || Boolean(message.reasoning) || message.toolSteps.length > 0;
    return hasBody ? "in_progress" : "queued";
  }
  return "completed";
}

function buildAssistantContent(message: AssistantChatMessage): ContentItem[] | string {
  const items: ContentItem[] = [];

  if (message.reasoning) {
    items.push({
      type: "reasoning",
      status: message.reasoningStreaming ? "in_progress" : "completed",
      summary: [{ type: "summary_text", text: message.reasoning }]
    });
  }

  for (const step of message.toolSteps) {
    items.push({
      type: "function_call",
      name: step.tool,
      arguments: step.arguments ? JSON.stringify(step.arguments) : undefined,
      status: toolStepStatus(step.status)
    });
    if (step.result) {
      items.push({
        type: "message",
        status: toolStepStatus(step.status),
        content: [{ type: "output_text", text: step.result }]
      });
    }
  }

  if (items.length === 0) {
    return message.content || (message.streaming ? "" : "");
  }

  if (message.content) {
    items.push({
      type: "message",
      status: message.streaming ? "in_progress" : "completed",
      content: [{ type: "output_text", text: message.content }]
    });
  }

  return items;
}

function mapAssistantMessage(message: AssistantChatMessage, createdAt: number): Message {
  const content = buildAssistantContent(message);

  return {
    id: message.id,
    role: "assistant",
    name: "AI 助手",
    content,
    createdAt,
    status: assistantDialogueStatus(message)
  };
}

export function mapChatMessagesToDialogue(messages: ChatMessage[]): Message[] {
  const baseTime = Date.now();

  return messages.map((message, index) => {
    const createdAt = baseTime - (messages.length - index) * 1000;

    if (message.role === "user") {
      return {
        id: message.id,
        role: "user",
        name: "我",
        content: message.text,
        createdAt,
        status: "completed"
      };
    }

    return mapAssistantMessage(message, createdAt);
  });
}
