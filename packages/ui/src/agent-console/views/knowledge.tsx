"use client";

import { useCallback, useEffect, useState } from "react";
import {
  Banner,
  Button,
  Col,
  Empty,
  List,
  Row,
  Space,
  Spin,
  Tabs,
  Typography
} from "@douyinfe/semi-ui-19";
import { IconBookOpenStroked, IconUpload } from "@douyinfe/semi-icons";

import {
  getAgentKnowledgeGraph,
  listAgentKnowledge,
  pickAndUploadKnowledge,
  readAgentKnowledge,
  type AgentKnowledgeFile,
  type AgentKnowledgeGraphLink,
  type AgentKnowledgeGraphNode
} from "@supportflow/shared/tauri-bridge/cmd/agent";

import { ViewShell } from "../shared/console-brand";
import { KnowledgeGraphPanel } from "./knowledge-graph-panel";

const { Text, Paragraph } = Typography;
const { TabPane } = Tabs;

type KnowledgeTab = "docs" | "graph";

function statusBannerType(tone: "success" | "error" | "info"): "success" | "danger" | "info" {
  if (tone === "success") return "success";
  if (tone === "error") return "danger";
  return "info";
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

  const toolbarExtra = (
    <Space spacing="tight">
      <Button icon={<IconUpload />} loading={uploading} onClick={() => void handleUploadClick()}>
        {uploading ? "正在导入…" : "上传文档"}
      </Button>
      <Tabs type="button" activeKey={tab} onChange={(key) => setTab(key as KnowledgeTab)}>
        <TabPane tab="文档" itemKey="docs" />
        <TabPane tab="图谱" itemKey="graph" />
      </Tabs>
    </Space>
  );

  return (
    <ViewShell
      title="知识库"
      description={statusMessage ? undefined : "浏览和探索你的知识库"}
      extra={toolbarExtra}
    >
      {statusMessage ? (
        <Banner
          fullMode={false}
          bordered={false}
          closeIcon={null}
          type={statusBannerType(statusTone)}
          description={<span style={{ whiteSpace: "pre-wrap" }}>{statusMessage}</span>}
          style={{ marginBottom: 16 }}
        />
      ) : null}

      {tab === "docs" ? (
        <Row gutter={16} style={{ minHeight: 0, flex: 1 }}>
          <Col span={7} xs={24} lg={7}>
            <div
              style={{
                border: "1px solid var(--semi-color-border)",
                borderRadius: 12,
                overflow: "hidden",
                height: "100%"
              }}
            >
              <div
                style={{
                  padding: "8px 12px",
                  borderBottom: "1px solid var(--semi-color-border)",
                  fontSize: 12,
                  color: "var(--semi-color-text-2)"
                }}
              >
                文档 ({files.length})
              </div>
              <div style={{ maxHeight: "calc(100vh - 280px)", overflowY: "auto", padding: 8 }}>
                {loading ? (
                  <Spin style={{ display: "block", margin: "24px auto" }} tip="加载知识库中…" />
                ) : (
                  <List
                    split
                    dataSource={files}
                    emptyContent={
                      <Empty description="暂无知识文档。点击「上传文档」导入 PDF、Word 等，或在工作区 knowledge/ 添加 Markdown。">
                        <Button
                          icon={<IconUpload />}
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
                        />
                      );
                    }}
                  />
                )}
              </div>
            </div>
          </Col>
          <Col span={17} xs={24} lg={17}>
            <div
              style={{
                border: "1px solid var(--semi-color-border)",
                borderRadius: 12,
                overflow: "hidden",
                minHeight: 320,
                display: "flex",
                flexDirection: "column",
                height: "100%"
              }}
            >
              {activePath ? (
                <>
                  <div
                    style={{
                      padding: "8px 16px",
                      borderBottom: "1px solid var(--semi-color-border)",
                      fontSize: 12,
                      color: "var(--semi-color-text-2)"
                    }}
                  >
                    <Text type="tertiary" size="small" code>
                      {activePath}
                    </Text>
                  </div>
                  <Paragraph
                    copyable
                    style={{
                      flex: 1,
                      margin: 0,
                      padding: 16,
                      overflow: "auto",
                      whiteSpace: "pre-wrap",
                      maxHeight: "calc(100vh - 300px)"
                    }}
                  >
                    {content || "从左侧选择文档查看"}
                  </Paragraph>
                </>
              ) : (
                <Empty
                  style={{ margin: "auto", padding: 32 }}
                  image={<IconBookOpenStroked size="extra-large" />}
                  title="从左侧选择文档查看"
                >
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
            </div>
          </Col>
        </Row>
      ) : (
        <KnowledgeGraphPanel
          nodes={graphNodes}
          links={graphLinks}
          loading={graphLoading}
          uploading={uploading}
          onUpload={handleUploadClick}
        />
      )}
    </ViewShell>
  );
}
