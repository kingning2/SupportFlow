"use client";

import { useCallback, useEffect, useState } from "react";
import { Spin, Typography } from "@douyinfe/semi-ui-19";

import { newAgentSession } from "@supportflow/shared/tauri-bridge/cmd/agent";
import { LocalCacheKey } from "@supportflow/shared/tauri-bridge/enums";
import { Chat } from "@supportflow/ui/agent-console/chat/chat";
import { useAgentConsoleState } from "@supportflow/ui/agent-console/hooks/use-agent-console-state";

import { WEWORK_USER_CHAT_PROMPTS } from "../constants/ai-chat-prompts";
import { useActiveWeworkAccount } from "../hooks/use-active-wework-account";
import { useWeworkConsoleContext } from "../wework-console-context";

const { Text } = Typography;

export function AiChat() {
  const { connectionStatus } = useWeworkConsoleContext();
  const { account } = useActiveWeworkAccount(connectionStatus);
  const { state, loading, error } = useAgentConsoleState();
  const [sessionId, setSessionId] = useState<string | undefined>(state?.sessionId);

  useEffect(() => {
    if (state?.sessionId) {
      setSessionId(state.sessionId);
    }
  }, [state?.sessionId]);

  const handleNewSession = useCallback(async () => {
    const nextId = await newAgentSession();
    setSessionId(nextId);
    localStorage.setItem(LocalCacheKey.AgentSessionId, nextId);
  }, []);

  if (loading && !state) {
    return (
      <div
        style={{ display: "flex", height: "100%", alignItems: "center", justifyContent: "center" }}
      >
        <Spin tip="正在加载 AI 助手…" />
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

  const topBanner =
    connectionStatus === "ready" && account
      ? {
          type: "info" as const,
          description: `当前企微账号：${account.label}${account.weworkUserId ? `（${account.weworkUserId}）` : ""}`
        }
      : connectionStatus !== "ready"
        ? {
            type: "warning" as const,
            description:
              "企微尚未连接。本助手仅回答账号与配置相关问题，不会查询客户对话等业务数据。"
          }
        : undefined;

  return (
    <div className="agent-chat-page">
      <Chat
        sessionId={sessionId}
        consoleState={state}
        onNewSession={() => {
          void handleNewSession();
        }}
        topBanner={topBanner}
        welcomePrompts={WEWORK_USER_CHAT_PROMPTS}
      />
    </div>
  );
}
