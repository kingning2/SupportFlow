"use client";

import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  UploadOutlined,
  FolderOutlined,
  ShareAltOutlined,
  FileTextOutlined,
  DeleteOutlined
} from "@ant-design/icons";
import {
  Button,
  Tabs,
  List,
  Card,
  Empty,
  Spin,
  Typography,
  Alert,
  ConfigProvider,
  Space,
  theme
} from "antd";

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

const { Title, Text } = Typography;

type KnowledgeTab = "docs" | "graph";

const WEWORK_PRIMARY = "#07C160";

export function KnowledgeView() {
  const { t } = useTranslation("console");
  const [tab, setTab] = useState<KnowledgeTab>("docs");
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [status, setStatus] = useState<{
    message: string;
    type: "success" | "error" | "info";
  } | null>(null);

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

  const clearStatus = () => setStatus(null);

  // Trigger Rust-side native file dialog. Rust will read the files from disk directly
  // and perform the ingest (MarkItDown first if available).
  const handleUploadClick = async () => {
    setUploading(true);
    setStatus({ message: t("knowledge_uploading"), type: "info" });

    try {
      const result = await pickAndUploadKnowledge("uploads");

      if (result.count > 0) {
        let msg = t("knowledge_upload_success", { count: result.count });
        if (result.memorySynced) {
          msg += ` · ${t("knowledge_upload_memory_synced")}`;
        }
        setStatus({ message: msg, type: "success" });
        await loadFiles();
        const first = result.results[0]?.path;
        if (first) {
          await openFile(first);
        }
      } else {
        setStatus({ message: t("knowledge_upload_failed"), type: "error" });
      }

      if (result.errors.length > 0) {
        const partial = t("knowledge_upload_partial", { count: result.errors.length });
        const detail = result.errors.map((e) => `${e.file}: ${e.message}`).join("\n");
        setStatus((prev) => {
          const base = prev ? `${prev.message}\n` : "";
          return {
            message: `${base}${partial}\n${detail}`,
            type: result.count > 0 ? "success" : "error"
          };
        });
      }
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      setStatus({
        message: `${t("knowledge_upload_failed")}: ${detail}`,
        type: "error"
      });
    } finally {
      setUploading(false);
    }
  };

  const docsContent = (
    <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[300px,1fr]">
      <Card
        size="small"
        title={
          <span>
            <FolderOutlined className="mr-2" />
            {t("knowledge_tab_docs")} ({files.length})
          </span>
        }
        styles={{
          body: {
            padding: 0,
            overflow: "auto"
          }
        }}
        className="h-full"
      >
        {loading ? (
          <div className="flex items-center justify-center p-8">
            <Spin />
          </div>
        ) : files.length === 0 ? (
          <div className="p-4">
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={t("knowledge_empty_hint")}>
              <Button
                size="small"
                icon={<UploadOutlined />}
                onClick={handleUploadClick}
                disabled={uploading}
              >
                {t("knowledge_upload_btn")}
              </Button>
            </Empty>
          </div>
        ) : (
          <List
            size="small"
            dataSource={files}
            renderItem={(file) => (
              <List.Item
                key={file.path}
                onClick={() => void openFile(file.path)}
                className="!cursor-pointer"
                style={{
                  background: activePath === file.path ? "rgba(7, 193, 96, 0.08)" : undefined,
                  borderLeft: activePath === file.path ? `3px solid ${WEWORK_PRIMARY}` : undefined
                }}
                actions={[
                  <Button
                    size="small"
                    key="remove"
                    icon={<DeleteOutlined />}
                    onClick={async () => {
                      await removeAgentKnowledge(file.path);
                      if (activePath === file.path) {
                        setActivePath(null);
                        setContent("");
                      }
                      await loadFiles();
                    }}
                    disabled={loading}
                  >
                    {t("knowledge_remove_btn")}
                  </Button>
                ]}
              >
                <List.Item.Meta
                  avatar={<FileTextOutlined style={{ color: "#666" }} />}
                  title={<span className="text-sm">{file.title}</span>}
                  description={<span className="font-mono text-xs text-gray-400">{file.path}</span>}
                />
              </List.Item>
            )}
          />
        )}
      </Card>

      <Card
        size="small"
        title={
          activePath ? (
            <span className="font-mono text-xs text-gray-500">{activePath}</span>
          ) : (
            t("knowledge_select_hint")
          )
        }
        className="flex h-full flex-col"
        styles={{
          body: {
            flex: 1,
            display: "flex",
            flexDirection: "column",
            padding: 0
          }
        }}
      >
        {activePath ? (
          <pre
            className="flex-1 overflow-auto p-4 text-sm whitespace-pre-wrap text-gray-700 dark:text-gray-200"
            style={{ margin: 0, fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace" }}
          >
            {content || t("knowledge_select_hint")}
          </pre>
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center">
            <FileTextOutlined style={{ fontSize: 32, color: "#bbb" }} />
            <Text type="secondary">{t("knowledge_select_hint")}</Text>
            <Button
              size="small"
              icon={<UploadOutlined />}
              onClick={handleUploadClick}
              disabled={uploading}
            >
              {t("knowledge_upload_btn")}
            </Button>
          </div>
        )}
      </Card>
    </div>
  );

  const graphContent = (
    <Card size="small" className="h-full">
      {graphLoading ? (
        <div className="flex justify-center p-12">
          <Spin />
        </div>
      ) : graphNodes.length === 0 ? (
        <Empty description={t("knowledge_empty_hint")}>
          <Button
            size="small"
            icon={<UploadOutlined />}
            onClick={handleUploadClick}
            disabled={uploading}
          >
            {t("knowledge_upload_btn")}
          </Button>
        </Empty>
      ) : (
        <div className="grid gap-6 lg:grid-cols-2">
          <div>
            <div className="mb-2 text-sm font-medium">
              {t("knowledge_graph_nodes")} ({graphNodes.length})
            </div>
            <List
              size="small"
              bordered
              dataSource={graphNodes}
              renderItem={(node) => (
                <List.Item>
                  <Text strong>{node.label}</Text>
                  <Text type="secondary" className="ml-2 font-mono text-xs">
                    {node.category}
                  </Text>
                </List.Item>
              )}
              style={{ maxHeight: "50vh", overflow: "auto" }}
            />
          </div>
          <div>
            <div className="mb-2 text-sm font-medium">
              {t("knowledge_graph_links")} ({graphLinks.length})
            </div>
            <List
              size="small"
              bordered
              dataSource={graphLinks}
              renderItem={(link, idx) => (
                <List.Item key={idx}>
                  <Text code>
                    {link.source} → {link.target}
                  </Text>
                </List.Item>
              )}
              style={{ maxHeight: "50vh", overflow: "auto" }}
            />
          </div>
        </div>
      )}
    </Card>
  );

  return (
    <ConfigProvider
      theme={{
        algorithm: theme.defaultAlgorithm,
        token: {
          colorPrimary: WEWORK_PRIMARY,
          borderRadius: 6
        }
      }}
    >
      <div className="flex h-full min-h-0 flex-col overflow-hidden bg-[var(--main-window-bg)]">
        <div className="border-b border-gray-200 bg-white px-6 py-3">
          <div className="flex items-center justify-between">
            <div>
              <Title level={5} style={{ margin: 0 }}>
                {t("knowledge_title")}
              </Title>
              <Text type="secondary" style={{ fontSize: 12 }}>
                {t("knowledge_desc")}
              </Text>
            </div>

            <Space>
              <Button icon={<UploadOutlined />} onClick={handleUploadClick} disabled={uploading}>
                {t("knowledge_upload_btn")}
              </Button>
            </Space>
          </div>
        </div>

        <div className="min-h-0 flex-1 overflow-auto p-6">
          {/* {statusAlert} */}

          <Tabs
            activeKey={tab}
            onChange={(k) => setTab(k as KnowledgeTab)}
            items={[
              {
                key: "docs",
                label: (
                  <span>
                    <FolderOutlined className="mr-1" />
                    {t("knowledge_tab_docs")}
                  </span>
                ),
                children: docsContent
              },
              {
                key: "graph",
                label: (
                  <span>
                    <ShareAltOutlined className="mr-1" />
                    {t("knowledge_tab_graph")}
                  </span>
                ),
                children: graphContent
              }
            ]}
            style={{ height: "100%" }}
          />
        </div>
      </div>
    </ConfigProvider>
  );
}
