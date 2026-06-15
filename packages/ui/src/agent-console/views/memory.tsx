"use client";

import { useEffect, useState } from "react";
import { Button, Card, Empty, Space, Spin, Table, Tabs, Typography } from "@douyinfe/semi-ui-19";
import { IconBookmark, IconMoon } from "@douyinfe/semi-icons";

import {
  listAgentMemory,
  readAgentMemory,
  type AgentMemoryItem
} from "@supportflow/shared/tauri-bridge/cmd/agent";

import { ViewShell } from "../shared/console-brand";

const { Text, Paragraph } = Typography;
const { TabPane } = Tabs;

export function Memory() {
  const [loading, setLoading] = useState(true);
  const [items, setItems] = useState<AgentMemoryItem[]>([]);
  const [activeTab, setActiveTab] = useState<"files" | "dreams">("files");
  const [activeFilename, setActiveFilename] = useState<string | null>(null);
  const [activeContent, setActiveContent] = useState("");

  useEffect(() => {
    let mounted = true;
    const load = async () => {
      try {
        const data = await listAgentMemory();
        if (mounted) setItems(data);
      } catch {
        if (mounted) setItems([]);
      } finally {
        if (mounted) setLoading(false);
      }
    };
    void load();
    return () => {
      mounted = false;
    };
  }, []);

  const filtered = items.filter((item) =>
    activeTab === "dreams" ? item.type === "dream" : item.type !== "dream"
  );

  const openFile = async (filename: string) => {
    try {
      const result = await readAgentMemory(filename);
      setActiveFilename(result.filename);
      setActiveContent(result.content);
    } catch {
      setActiveFilename(filename);
      setActiveContent("");
    }
  };

  return (
    <ViewShell title="记忆管理" description="查看 Agent 记忆文件和内容">
      <Tabs
        type="button"
        activeKey={activeTab}
        onChange={(k) => setActiveTab(k as "files" | "dreams")}
      >
        <TabPane tab="记忆文件" itemKey="files" icon={<IconBookmark />} />
        <TabPane tab="梦境日记" itemKey="dreams" icon={<IconMoon />} />
      </Tabs>

      <div style={{ marginTop: 16 }}>
        {activeFilename ? (
          <Card
            title={<Text code>{activeFilename}</Text>}
            footer={
              <Button theme="light" type="tertiary" onClick={() => setActiveFilename(null)}>
                返回列表
              </Button>
            }
          >
            <Paragraph
              style={{ margin: 0, whiteSpace: "pre-wrap", maxHeight: "60vh", overflow: "auto" }}
            >
              {activeContent || "暂无记忆内容"}
            </Paragraph>
          </Card>
        ) : loading ? (
          <Spin tip="记忆文件将显示在此处" style={{ display: "block", margin: "48px auto" }} />
        ) : filtered.length === 0 ? (
          <Empty
            image={<IconBookmark size="extra-large" />}
            title="记忆管理"
            description="暂无记忆内容"
          />
        ) : (
          <Table
            pagination={false}
            onRow={(record) => ({
              onClick: () => {
                if (record?.filename) void openFile(record.filename);
              },
              style: { cursor: "pointer" }
            })}
            columns={[
              { title: "文件名", dataIndex: "filename" },
              { title: "类型", dataIndex: "type" },
              { title: "大小", dataIndex: "size" },
              { title: "更新时间", dataIndex: "updatedAt" }
            ]}
            dataSource={filtered.map((item) => ({ ...item, key: item.filename }))}
          />
        )}
      </div>
    </ViewShell>
  );
}
