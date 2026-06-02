/** Agent stream chunk `type` values (mirrors Rust `map_stream_event`). */
export enum AgentStreamChunkType {
  Reasoning = "reasoning",
  Delta = "delta",
  ToolStart = "tool_start",
  ToolEnd = "tool_end",
  Cancelled = "cancelled",
  Done = "done"
}

export const AGENT_STREAM_CHUNK_TYPE_VALUES = Object.values(
  AgentStreamChunkType
) as AgentStreamChunkType[];

export function isAgentStreamChunkType(value: string): value is AgentStreamChunkType {
  return (AGENT_STREAM_CHUNK_TYPE_VALUES as string[]).includes(value);
}
