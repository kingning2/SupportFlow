"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { IconChevronLeft } from "@douyinfe/semi-icons";
import { IconButton, MCPConfigure } from "@douyinfe/semi-ui-19";
import type { MCPOption } from "@douyinfe/semi-foundation/lib/es/sidebar/mcpCofContentFoundation";

import type { AgentConsoleState } from "@supportflow/shared/contracts";

import { readStoredMcps, writeStoredMcps } from "./chat-mcp-storage";

const SIDER_COLLAPSED_WIDTH = 40;

function toMcpOptions(mcpStatus: Record<string, string>, enabled: string[]): MCPOption[] {
  return Object.entries(mcpStatus).map(([name, status]) => ({
    label: name,
    value: name,
    desc: status,
    active: enabled.includes(name)
  }));
}

interface ChatMcpSidebarProps {
  sessionId?: string;
  consoleState: AgentConsoleState | null;
  expanded: boolean;
  onExpand: () => void;
  onCollapse: () => void;
}

export function ChatMcpSidebar({
  sessionId,
  consoleState,
  expanded,
  onExpand,
  onCollapse
}: ChatMcpSidebarProps) {
  const mcpStatus = consoleState?.mcpStatus ?? {};
  const mcpNames = useMemo(() => Object.keys(mcpStatus), [mcpStatus]);
  const [enabledMcps, setEnabledMcps] = useState<string[]>(() =>
    readStoredMcps(sessionId, mcpNames)
  );

  useEffect(() => {
    setEnabledMcps(readStoredMcps(sessionId, mcpNames));
  }, [sessionId, mcpNames.join("|")]);

  const options = useMemo(() => toMcpOptions(mcpStatus, enabledMcps), [enabledMcps, mcpStatus]);

  const handleStatusChange = useCallback(
    (nextOptions: MCPOption[]) => {
      const enabled = nextOptions
        .filter((item) => item.active)
        .map((item) => item.value ?? item.label ?? "")
        .filter(Boolean);
      setEnabledMcps(enabled);
      writeStoredMcps(sessionId, enabled);
    },
    [sessionId]
  );

  if (!expanded) {
    return (
      <div
        className="agent-chat-sidebar-collapsed"
        style={{ height: "100%", width: SIDER_COLLAPSED_WIDTH }}
      >
        <IconButton
          icon={<IconChevronLeft />}
          type="tertiary"
          aria-label="展开 MCP 配置"
          onClick={onExpand}
          style={{ flex: 1, height: "100%", borderRadius: 0 }}
        />
      </div>
    );
  }

  return (
    <MCPConfigure
      className="agent-chat-mcp-configure"
      title="MCP 配置"
      visible
      motion={false}
      resizable
      showClose
      defaultSize={{ width: 320 }}
      style={{ height: "100%" }}
      options={options}
      onStatusChange={handleStatusChange}
      onCancel={onCollapse}
    />
  );
}
