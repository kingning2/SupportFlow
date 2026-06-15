"use client";

import { useEffect, useState } from "react";
import { Card, Empty, Space, Spin, Tag, Typography } from "@douyinfe/semi-ui-19";
import { IconClock } from "@douyinfe/semi-icons";

import { listAgentTasks, type AgentTaskSummary } from "@supportflow/shared/tauri-bridge/cmd/agent";

import { ViewShell } from "../shared/console-brand";

const { Text } = Typography;

export function Tasks() {
  const [loading, setLoading] = useState(true);
  const [tasks, setTasks] = useState<AgentTaskSummary[]>([]);

  useEffect(() => {
    let mounted = true;
    const load = async () => {
      try {
        const data = await listAgentTasks();
        if (mounted) {
          setTasks(data.filter((task) => task.enabled !== false));
        }
      } catch {
        if (mounted) setTasks([]);
      } finally {
        if (mounted) setLoading(false);
      }
    };
    void load();
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <ViewShell title="定时任务" description="查看和管理定时任务">
      {loading ? (
        <Spin tip="加载定时任务中..." style={{ display: "block", margin: "48px auto" }} />
      ) : tasks.length === 0 ? (
        <Empty
          image={<IconClock size="extra-large" />}
          title="定时任务"
          description="暂无定时任务"
        />
      ) : (
        <Space vertical style={{ width: "100%" }} spacing="medium">
          {tasks.map((task) => (
            <Card key={task.id} bodyStyle={{ padding: 16 }}>
              <Space>
                <Tag color="green" size="small">
                  启用
                </Tag>
                <Text strong>{task.name || task.id}</Text>
              </Space>
              <Text type="tertiary" size="small" style={{ display: "block", marginTop: 8 }}>
                下次执行: {task.nextRunAt ?? "—"}
              </Text>
            </Card>
          ))}
        </Space>
      )}
    </ViewShell>
  );
}
