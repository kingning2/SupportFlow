"use client";

import { Spin, Typography } from "@douyinfe/semi-ui-19";

import { ProviderSettings } from "@supportflow/ui/agent-console/views/provider-settings";
import { useAgentConsoleState } from "@supportflow/ui/agent-console/hooks/use-agent-console-state";

const { Text } = Typography;

export function AiConfig() {
  const { state, setState, loading, error } = useAgentConsoleState();

  if (loading && !state) {
    return (
      <div className="wework-ai-config wework-ai-config--loading">
        <Spin tip="正在加载配置…" />
      </div>
    );
  }

  if (error && !state) {
    return (
      <div className="wework-ai-config wework-ai-config--error">
        <Text type="danger">{error}</Text>
      </div>
    );
  }

  return (
    <div className="wework-ai-config agent-console-page-enter agent-console-interactive">
      <ProviderSettings state={state} onRefresh={setState} embedded />
    </div>
  );
}
