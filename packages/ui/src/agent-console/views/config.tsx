"use client";

import { Col, Descriptions, Row, Typography } from "@douyinfe/semi-ui-19";

import type { AgentConsoleState } from "@supportflow/shared/contracts";

import { MutedHint, SectionCard, ViewShell } from "../shared/console-brand";

const { Text } = Typography;

export function RuntimeConfigPanel({ state }: { state: AgentConsoleState | null }) {
  return (
    <>
      <Row gutter={24}>
        <Col span={12}>
          <SectionCard title="路径">
            <Descriptions
              align="left"
              row
              data={[
                {
                  key: "工作区",
                  value: (
                    <Text code style={{ wordBreak: "break-all" }}>
                      {state?.workspaceDir ?? "—"}
                    </Text>
                  )
                },
                {
                  key: "配置源 (resources)",
                  value: (
                    <Text code style={{ wordBreak: "break-all" }}>
                      {state?.configPath ?? "—"}
                    </Text>
                  )
                }
              ]}
            />
          </SectionCard>
        </Col>
        <Col span={12}>
          <SectionCard title="采样参数">
            <Descriptions
              align="left"
              row
              data={[
                { key: "temperature", value: state?.temperature ?? "默认" },
                { key: "top_p", value: state?.topP ?? "默认" },
                { key: "请求超时", value: state?.requestTimeout ?? "默认" }
              ]}
            />
          </SectionCard>
        </Col>
      </Row>

      {state?.mcpStatus && Object.keys(state.mcpStatus).length > 0 ? (
        <SectionCard title="MCP" style={{ marginTop: 16 }}>
          <Descriptions
            align="left"
            row
            data={Object.entries(state.mcpStatus).map(([name, status]) => ({
              key: name,
              value: status
            }))}
          />
        </SectionCard>
      ) : null}
    </>
  );
}

export function Config({ state }: { state: AgentConsoleState | null }) {
  return (
    <ViewShell
      className="agent-console-interactive agent-console-page-enter"
      title="运行配置"
      description="工作区路径、采样参数与 MCP；模型厂商请在「供应商配置」中管理。"
    >
      <RuntimeConfigPanel state={state} />
      <MutedHint>
        配置源文件：src-tauri/resources/config.json（随 Tauri 打包）。修改后需重启应用。
      </MutedHint>
    </ViewShell>
  );
}
