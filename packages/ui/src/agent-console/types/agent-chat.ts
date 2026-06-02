/** Frontend chat message model (Tauri stream → UI). */

export type ToolStepStatus = "running" | "success" | "error";

export interface ToolStep {
  id: string;
  tool: string;
  arguments?: Record<string, unknown>;
  status: ToolStepStatus;
  result?: string;
  executionTime?: number;
}

export interface UserChatMessage {
  id: string;
  role: "user";
  text: string;
}

export interface AssistantChatMessage {
  id: string;
  role: "assistant";
  requestId: string;
  reasoning: string;
  reasoningStreaming: boolean;
  toolSteps: ToolStep[];
  content: string;
  streaming: boolean;
  cancelled: boolean;
}

export type ChatMessage = UserChatMessage | AssistantChatMessage;

export function isAssistantMessage(msg: ChatMessage): msg is AssistantChatMessage {
  return msg.role === "assistant";
}
