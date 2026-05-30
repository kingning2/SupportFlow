import { AgentStreamChunkType } from "@/enums/agent-stream-chunk-type";
import type { AgentStreamChunk } from "@/generated/contracts";
import type { AssistantChatMessage, ChatMessage, ToolStep } from "@/types/agent-chat";

function asRecord(value: unknown): Record<string, unknown> | undefined {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return undefined;
}

function findAssistant(
  messages: ChatMessage[],
  requestId: string
): AssistantChatMessage | undefined {
  const msg = messages.find((m) => m.role === "assistant" && m.requestId === requestId);
  return msg?.role === "assistant" ? msg : undefined;
}

export function createAssistantMessage(requestId: string): AssistantChatMessage {
  return {
    id: requestId,
    role: "assistant",
    requestId,
    reasoning: "",
    reasoningStreaming: false,
    toolSteps: [],
    content: "",
    streaming: true,
    cancelled: false
  };
}

export function applyStreamChunk(messages: ChatMessage[], chunk: AgentStreamChunk): ChatMessage[] {
  const next = [...messages];
  const idx = next.findIndex((m) => m.role === "assistant" && m.requestId === chunk.requestId);

  if (idx === -1) {
    return next;
  }

  const current = next[idx];
  if (current.role !== "assistant") {
    return next;
  }

  let assistant: AssistantChatMessage = { ...current };

  switch (chunk.type) {
    case AgentStreamChunkType.Reasoning:
      assistant = {
        ...assistant,
        reasoning: assistant.reasoning + (chunk.content ?? ""),
        reasoningStreaming: true
      };
      break;

    case AgentStreamChunkType.Delta:
      assistant = {
        ...assistant,
        reasoningStreaming: false,
        content: assistant.content + (chunk.content ?? "")
      };
      break;

    case AgentStreamChunkType.ToolStart: {
      const step: ToolStep = {
        id: `${chunk.requestId}-${assistant.toolSteps.length}`,
        tool: chunk.tool ?? "tool",
        arguments: asRecord(chunk.arguments),
        status: "running"
      };
      assistant = {
        ...assistant,
        reasoningStreaming: false,
        toolSteps: [...assistant.toolSteps, step]
      };
      break;
    }

    case AgentStreamChunkType.ToolEnd: {
      const steps = [...assistant.toolSteps];
      const lastIdx = steps.length - 1;
      if (lastIdx >= 0) {
        const last = steps[lastIdx];
        steps[lastIdx] = {
          ...last,
          status: chunk.status === "error" ? "error" : "success",
          result: chunk.result,
          executionTime: chunk.executionTime
        };
      }
      assistant = {
        ...assistant,
        toolSteps: steps
      };
      break;
    }

    case AgentStreamChunkType.Cancelled:
      assistant = {
        ...assistant,
        streaming: false,
        reasoningStreaming: false,
        cancelled: true
      };
      break;

    case AgentStreamChunkType.Done:
      assistant = {
        ...assistant,
        streaming: false,
        reasoningStreaming: false,
        content: chunk.content ?? assistant.content
      };
      break;

    default:
      break;
  }

  next[idx] = assistant;
  return next;
}

export function finalizeAssistant(
  messages: ChatMessage[],
  requestId: string,
  error?: string
): ChatMessage[] {
  const assistant = findAssistant(messages, requestId);
  if (!assistant) {
    return messages;
  }

  const next = [...messages];
  const idx = next.findIndex((m) => m.role === "assistant" && m.requestId === requestId);
  if (idx === -1) {
    return next;
  }

  next[idx] = {
    ...assistant,
    streaming: false,
    reasoningStreaming: false,
    content: error ? `${assistant.content}\n\n**${error}**`.trim() : assistant.content
  };
  return next;
}
