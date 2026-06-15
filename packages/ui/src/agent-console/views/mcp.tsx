"use client";

import { Descriptions, Empty, Space, Tag, Typography } from "@douyinfe/semi-ui-19";
import { IconRefresh } from "@douyinfe/semi-icons";
import { Button } from "@douyinfe/semi-ui-19";

import type { AgentConsoleState } from "@supportflow/shared/contracts";

import { MutedHint, SectionCard, ViewShell } from "../shared/console-brand";

const { Text } = Typography;

interface McpProps {
  state: AgentConsoleState | null;
  onRefresh?: () => void;
}

function statusColor(status: string): "green" | "red" | "grey" | "blue" {
  const lower = status.toLowerCase();
  if (lower.includes("ready") || lower.includes("connected") || lower.includes("ok")) {
    return "green";
  }
  if (lower.includes("fail") || lower.includes("error")) {
    return "red";
  }
  if (lower.includes("load") || lower.includes("start")) {
    return "blue";
  }
  return "grey";
}

export function Mcp({ state, onRefresh }: McpProps) {
  const entries = Object.entries(state?.mcpStatus ?? {});
  const mcpJsonPath = state?.workspaceDir ? `${state.workspaceDir}/mcp.json` : "工作区/mcp.json";

  return (
    <ViewShell
      title="MCP 服务"
      description="查看 Model Context Protocol 服务加载状态；配置在工作区 mcp.json。"
      extra={
        onRefresh ? (
          <Button icon={<IconRefresh />} theme="light" type="tertiary" onClick={onRefresh}>
            刷新
          </Button>
        ) : null
      }
    >
      <SectionCard title="配置文件" style={{ marginBottom: 24 }}>
        <Descriptions
          align="left"
          row
          data={[
            {
              key: "mcp.json",
              value: (
                <Text code style={{ wordBreak: "break-all" }}>
                  {mcpJsonPath}
                </Text>
              )
            },
            {
              key: "格式",
              value: "Claude / Cursor 风格 `mcpServers` 对象，或内部列表格式"
            }
          ]}
        />
      </SectionCard>

      <SectionCard title="服务状态">
        {entries.length === 0 ? (
          <Empty description="尚未加载 MCP 服务。在工作区创建 mcp.json 后重启或刷新 Agent 状态。" />
        ) : (
          <Space vertical align="start" spacing="medium" style={{ width: "100%" }}>
            {entries.map(([name, status]) => (
              <Space key={name} wrap>
                <Text strong>{name}</Text>
                <Tag color={statusColor(status)}>{status}</Tag>
              </Space>
            ))}
          </Space>
        )}
      </SectionCard>

      <div style={{ marginTop: 24 }}>
        <MutedHint>
          MCP 工具会出现在「技能与工具」页的工具列表中（标记为 MCP）。修改 mcp.json
          后需重启应用或重新加载 Agent 运行时。
        </MutedHint>
      </div>
    </ViewShell>
  );
}
