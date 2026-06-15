"use client";

import { Spin, Typography } from "@douyinfe/semi-ui-19";

import { Mcp } from "@supportflow/ui/agent-console/views/mcp";
import { useAgentConsoleState } from "@supportflow/ui/agent-console/hooks/use-agent-console-state";

const { Text } = Typography;

export function McpPage() {
  const { state, loading, error, reload } = useAgentConsoleState();

  if (loading && !state) {
    return (
      <div style={{ display: "flex", justifyContent: "center", padding: 48 }}>
        <Spin tip="正在加载 MCP 状态…" />
      </div>
    );
  }

  if (error && !state) {
    return (
      <div style={{ padding: 24 }}>
        <Text type="danger">{error}</Text>
      </div>
    );
  }

  return (
    <Mcp
      state={state}
      onRefresh={() => {
        void reload();
      }}
    />
  );
}
