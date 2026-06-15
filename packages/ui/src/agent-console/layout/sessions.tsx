"use client";

import { useEffect, useState } from "react";
import { Button, Empty, List, SideSheet, Space, Spin, Typography } from "@douyinfe/semi-ui-19";
import { IconPlus } from "@douyinfe/semi-icons";

import {
  listAgentSessions,
  type AgentSessionSummary
} from "@supportflow/shared/tauri-bridge/cmd/agent";

const { Text } = Typography;

interface SessionsProps {
  open: boolean;
  sessionId?: string;
  onClose: () => void;
  onNewChat: () => void;
}

export function Sessions({ open, sessionId, onClose, onNewChat }: SessionsProps) {
  const [loading, setLoading] = useState(false);
  const [sessions, setSessions] = useState<AgentSessionSummary[]>([]);

  useEffect(() => {
    if (!open) {
      return;
    }
    let mounted = true;
    const load = async () => {
      setLoading(true);
      try {
        const data = await listAgentSessions();
        if (mounted) {
          setSessions(data);
        }
      } catch {
        if (mounted) {
          setSessions([]);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };
    void load();
    return () => {
      mounted = false;
    };
  }, [open, sessionId]);

  return (
    <SideSheet
      visible={open}
      placement="left"
      width={280}
      title="历史会话"
      headerStyle={{ borderBottom: "1px solid var(--semi-color-border)" }}
      bodyStyle={{ padding: 12 }}
      onCancel={onClose}
      closable
    >
      <Space vertical style={{ width: "100%" }} spacing="medium">
        <Button block icon={<IconPlus />} type="primary" onClick={onNewChat}>
          新对话
        </Button>

        {loading ? (
          <Spin style={{ display: "block", margin: "24px auto" }} />
        ) : sessions.length === 0 ? (
          <Empty description="暂无历史会话，发送消息或新建对话后会出现在这里。" />
        ) : (
          <List
            split={false}
            dataSource={sessions}
            renderItem={(session) => (
              <List.Item
                style={{
                  borderRadius: 8,
                  marginBottom: 8,
                  background:
                    session.id === sessionId
                      ? "var(--semi-color-primary-light-default)"
                      : "transparent"
                }}
                main={
                  <Space vertical align="start" spacing={2}>
                    <Text strong ellipsis style={{ maxWidth: "100%" }}>
                      {session.title || "新对话"}
                    </Text>
                    <Text type="tertiary" size="small" code ellipsis style={{ maxWidth: "100%" }}>
                      {session.id}
                    </Text>
                  </Space>
                }
              />
            )}
          />
        )}
      </Space>
    </SideSheet>
  );
}
