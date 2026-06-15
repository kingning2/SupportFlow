"use client";

import { useCallback, useEffect, useState } from "react";
import { Banner, Button, Empty, List, Space, Spin, Tabs, Typography } from "@douyinfe/semi-ui-19";
import { IconDelete, IconUpload } from "@douyinfe/semi-icons";

import {
  getAgentKnowledgeGraph,
  listAgentKnowledge,
  pickAndUploadKnowledge,
  readAgentKnowledge,
  removeAgentKnowledge,
  type AgentKnowledgeFile,
  type AgentKnowledgeGraphLink,
  type AgentKnowledgeGraphNode
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { KnowledgeGraphPanel } from "@supportflow/ui/agent-console/views/knowledge-graph-panel";

import {
  WeworkListPanel,
  WeworkMainPanel,
  WeworkPanelBody,
  WeworkPanelHeader,
  WeworkWorkspace,
  WeworkWorkspaceSplit
} from "../layout/workspace-layout";

const { Text, Title, Paragraph } = Typography;
const { TabPane } = Tabs;

type KnowledgeTab = "docs" | "graph";

function statusBannerType(tone: "success" | "error" | "info"): "success" | "danger" | "info" {
  if (tone === "success") return "success";
  if (tone === "error") return "danger";
  return "info";
}

function KnowledgeToolbar({
  tab,
  uploading,
  statusMessage,
  statusTone,
  onTabChange,
  onUpload
}: {
  tab: KnowledgeTab;
  uploading: boolean;
  statusMessage: string | null;
  statusTone: "success" | "error" | "info";
  onTabChange: (tab: KnowledgeTab) => void;
  onUpload: () => void;
}) {
  return (
    <WeworkPanelHeader
      style={{
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "space-between",
        gap: 12,
        flexWrap: "wrap"
      }}
    >
      <Space vertical align="start" spacing="tight" style={{ minWidth: 0, flex: 1 }}>
        <Title heading={6} style={{ margin: 0 }}>
          知识库
        </Title>
        {statusMessage ? (
          <Banner
            fullMode={false}
            bordered={false}
            closeIcon={null}
            type={statusBannerType(statusTone)}
            description={<span style={{ whiteSpace: "pre-wrap" }}>{statusMessage}</span>}
            style={{ padding: "4px 8px" }}
          />
        ) : (
          <Text type="tertiary" size="small">
            浏览和探索你的知识库
          </Text>
        )}
      </Space>
      <Space spacing="tight" style={{ flexShrink: 0 }}>
        <Button icon={<IconUpload />} theme="light" disabled={uploading} onClick={onUpload}>
          {uploading ? "正在导入…" : "上传文档"}
        </Button>
        <Tabs type="button" activeKey={tab} onChange={(key) => onTabChange(key as KnowledgeTab)}>
          <TabPane tab="文档" itemKey="docs" />
          <TabPane tab="图谱" itemKey="graph" />
        </Tabs>
      </Space>
    </WeworkPanelHeader>
  );
}

export function Knowledge() {
  const [tab, setTab] = useState<KnowledgeTab>("docs");
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [statusTone, setStatusTone] = useState<"success" | "error" | "info">("info");
  const [files, setFiles] = useState<AgentKnowledgeFile[]>([]);
  const [activePath, setActivePath] = useState<string | null>(null);
  const [content, setContent] = useState("");
  const [graphNodes, setGraphNodes] = useState<AgentKnowledgeGraphNode[]>([]);
  const [graphLinks, setGraphLinks] = useState<AgentKnowledgeGraphLink[]>([]);
  const [graphLoading, setGraphLoading] = useState(false);

  const loadFiles = useCallback(async () => {
    setLoading(true);
    try {
      const data = await listAgentKnowledge();
      setFiles(data);
    } catch {
      setFiles([]);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadGraph = useCallback(async () => {
    setGraphLoading(true);
    try {
      const graph = await getAgentKnowledgeGraph();
      setGraphNodes(graph.nodes);
      setGraphLinks(graph.links);
    } catch {
      setGraphNodes([]);
      setGraphLinks([]);
    } finally {
      setGraphLoading(false);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    queueMicrotask(() => {
      if (!cancelled) void loadFiles();
    });
    return () => {
      cancelled = true;
    };
  }, [loadFiles]);

  useEffect(() => {
    if (tab !== "graph") return;
    let cancelled = false;
    queueMicrotask(() => {
      if (!cancelled) void loadGraph();
    });
    return () => {
      cancelled = true;
    };
  }, [tab, loadGraph]);

  const openFile = async (path: string) => {
    setActivePath(path);
    try {
      const result = await readAgentKnowledge(path);
      setContent(result.content);
    } catch {
      setContent("");
    }
  };

  const handleUploadClick = async () => {
    setUploading(true);
    setStatusMessage("正在导入…");
    setStatusTone("info");

    try {
      const result = await pickAndUploadKnowledge("uploads");

      if (result.count > 0) {
        let msg = `已导入 ${result.count} 个文档`;
        if (result.memorySynced) {
          msg += " / 记忆索引已更新";
        }
        setStatusMessage(msg);
        setStatusTone("success");
        await loadFiles();
        const first = result.results[0]?.path;
        if (first) {
          await openFile(first);
        }
        if (tab === "graph") {
          await loadGraph();
        }
      } else {
        setStatusMessage("导入失败");
        setStatusTone("error");
      }

      if (result.errors.length > 0) {
        const partial = `${result.errors.length} 个文件导入失败`;
        const detail = result.errors.map((e) => `${e.file}: ${e.message}`).join("\n");
        setStatusMessage((prev) =>
          prev ? `${prev}\n${partial}\n${detail}` : `${partial}\n${detail}`
        );
        setStatusTone(result.count > 0 ? "success" : "error");
      }
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      setStatusMessage(`导入失败: ${detail}`);
      setStatusTone("error");
    } finally {
      setUploading(false);
    }
  };

  const handleRemove = async (path: string) => {
    await removeAgentKnowledge(path);
    if (activePath === path) {
      setActivePath(null);
      setContent("");
    }
    await loadFiles();
    if (tab === "graph") {
      await loadGraph();
    }
  };

  const toolbar = (
    <KnowledgeToolbar
      tab={tab}
      uploading={uploading}
      statusMessage={statusMessage}
      statusTone={statusTone}
      onTabChange={setTab}
      onUpload={() => void handleUploadClick()}
    />
  );

  if (tab === "graph") {
    return (
      <WeworkWorkspace>
        {toolbar}
        <WeworkMainPanel style={{ flex: 1 }}>
          <WeworkPanelBody style={{ padding: 16 }}>
            <KnowledgeGraphPanel
              nodes={graphNodes}
              links={graphLinks}
              loading={graphLoading}
              uploading={uploading}
              onUpload={handleUploadClick}
            />
          </WeworkPanelBody>
        </WeworkMainPanel>
      </WeworkWorkspace>
    );
  }

  return (
    <WeworkWorkspace>
      <WeworkWorkspaceSplit>
        <WeworkListPanel>
          <WeworkPanelHeader>
            <Title heading={6} style={{ margin: 0 }}>
              文档 ({files.length})
            </Title>
          </WeworkPanelHeader>
          <WeworkPanelBody style={{ padding: 0 }}>
            {loading ? (
              <Spin style={{ display: "block", margin: "24px auto" }} />
            ) : (
              <List
                split
                dataSource={files}
                emptyContent={
                  <Empty description="暂无知识文档。点击「上传文档」导入 PDF、Word 等，或在工作区 knowledge/ 添加 Markdown。">
                    <Button
                      icon={<IconUpload />}
                      theme="light"
                      disabled={uploading}
                      onClick={() => void handleUploadClick()}
                    >
                      上传文档
                    </Button>
                  </Empty>
                }
                renderItem={(file) => {
                  const isActive = activePath === file.path;
                  return (
                    <List.Item
                      onClick={() => void openFile(file.path)}
                      style={
                        isActive
                          ? {
                              backgroundColor: "var(--semi-color-primary-light-default)",
                              cursor: "pointer"
                            }
                          : { cursor: "pointer" }
                      }
                      main={
                        <Space vertical align="start" spacing={4} style={{ width: "100%" }}>
                          <Text strong>{file.title}</Text>
                          <Text type="tertiary" size="small" code>
                            {file.path}
                          </Text>
                        </Space>
                      }
                      extra={
                        <Button
                          icon={<IconDelete />}
                          type="tertiary"
                          theme="borderless"
                          size="small"
                          disabled={loading}
                          onClick={(e) => {
                            e.stopPropagation();
                            void handleRemove(file.path);
                          }}
                        >
                          删除
                        </Button>
                      }
                    />
                  );
                }}
              />
            )}
          </WeworkPanelBody>
        </WeworkListPanel>

        <WeworkMainPanel>
          {toolbar}
          <WeworkPanelBody style={{ padding: 0 }}>
            {activePath ? (
              <>
                <WeworkPanelHeader style={{ minHeight: "auto", padding: "8px 16px" }}>
                  <Text type="tertiary" size="small" code>
                    {activePath}
                  </Text>
                </WeworkPanelHeader>
                <Paragraph
                  copyable
                  style={{
                    flex: 1,
                    margin: 0,
                    padding: 16,
                    overflow: "auto",
                    whiteSpace: "pre-wrap"
                  }}
                >
                  {content || "从左侧选择文档查看"}
                </Paragraph>
              </>
            ) : (
              <Empty style={{ margin: "auto", padding: 32 }} title="从左侧选择文档查看">
                <Button
                  icon={<IconUpload />}
                  theme="light"
                  disabled={uploading}
                  onClick={() => void handleUploadClick()}
                >
                  上传文档
                </Button>
              </Empty>
            )}
          </WeworkPanelBody>
        </WeworkMainPanel>
      </WeworkWorkspaceSplit>
    </WeworkWorkspace>
  );
}
