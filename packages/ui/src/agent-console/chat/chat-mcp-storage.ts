const MCP_STORAGE_PREFIX = "agent-chat-mcp-enabled:";

export function readStoredMcps(sessionId: string | undefined, allNames: string[]): string[] {
  if (typeof window === "undefined" || allNames.length === 0) {
    return allNames;
  }
  const key = `${MCP_STORAGE_PREFIX}${sessionId ?? "default"}`;
  try {
    const raw = localStorage.getItem(key);
    if (!raw) {
      return allNames;
    }
    const parsed = JSON.parse(raw) as string[];
    return parsed.filter((name) => allNames.includes(name));
  } catch {
    return allNames;
  }
}

export function writeStoredMcps(sessionId: string | undefined, enabled: string[]) {
  if (typeof window === "undefined") {
    return;
  }
  const key = `${MCP_STORAGE_PREFIX}${sessionId ?? "default"}`;
  localStorage.setItem(key, JSON.stringify(enabled));
}

export function getChatEnabledMcps(
  sessionId: string | undefined,
  mcpStatus: Record<string, string>
) {
  return readStoredMcps(sessionId, Object.keys(mcpStatus));
}
