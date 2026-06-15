"use client";

import { Graphin } from "@antv/graphin";
import { CanvasEvent, type Graph, NodeEvent } from "@antv/g6";
import { useEffect, useMemo, useState } from "react";
import {
  Button,
  Card,
  Col,
  Empty,
  Input,
  List,
  Row,
  Space,
  Spin,
  Tag,
  Typography
} from "@douyinfe/semi-ui-19";
import {
  IconBolt,
  IconClear,
  IconRefresh,
  IconSearch,
  IconTreeTriangleDown
} from "@douyinfe/semi-icons";

import type {
  AgentKnowledgeGraphLink,
  AgentKnowledgeGraphNode
} from "@supportflow/shared/tauri-bridge/cmd/agent";

const { Text, Title, Paragraph } = Typography;

interface KnowledgeGraphPanelProps {
  links: AgentKnowledgeGraphLink[];
  loading: boolean;
  nodes: AgentKnowledgeGraphNode[];
  onUpload: () => Promise<void> | void;
  uploading: boolean;
}

interface GraphPanelNode extends AgentKnowledgeGraphNode {
  degree: number;
  neighbors: AgentKnowledgeGraphNode[];
}

interface GraphState {
  filteredNodeCount: number;
  options: Record<string, unknown>;
  selectedNode: GraphPanelNode | null;
}

type GraphNodeClickEvent = {
  target?: {
    id?: string | number;
  };
};

const GRAPH_THEME = {
  accent: "hsl(var(--primary))",
  accentSoft: "hsl(var(--primary) / 0.16)",
  canvas: "hsl(var(--surface-0, var(--background)))",
  edge: "hsl(var(--border) / 0.75)",
  edgeHighlight: "hsl(var(--primary) / 0.9)",
  label: "hsl(var(--foreground))",
  nodeFill: "hsl(var(--card))",
  nodeStroke: "hsl(var(--border))",
  nodeText: "hsl(var(--foreground))"
};

function buildGraphState(params: {
  links: AgentKnowledgeGraphLink[];
  nodes: AgentKnowledgeGraphNode[];
  query: string;
  selectedNodeId: string | null;
}): GraphState {
  const { links, nodes, query, selectedNodeId } = params;
  const normalizedQuery = query.trim().toLowerCase();
  const nodeMap = new Map<string, AgentKnowledgeGraphNode>();
  const adjacency = new Map<string, Set<string>>();
  const degrees = new Map<string, number>();

  for (const node of nodes) {
    nodeMap.set(node.id, node);
    adjacency.set(node.id, new Set());
    degrees.set(node.id, 0);
  }

  for (const link of links) {
    adjacency.get(link.source)?.add(link.target);
    adjacency.get(link.target)?.add(link.source);
    degrees.set(link.source, (degrees.get(link.source) ?? 0) + 1);
    degrees.set(link.target, (degrees.get(link.target) ?? 0) + 1);
  }

  const matchedIds = new Set(
    nodes
      .filter(
        (node) =>
          normalizedQuery.length === 0 ||
          node.label.toLowerCase().includes(normalizedQuery) ||
          node.id.toLowerCase().includes(normalizedQuery) ||
          node.category.toLowerCase().includes(normalizedQuery)
      )
      .map((node) => node.id)
  );

  const selectedNeighbors = selectedNodeId ? (adjacency.get(selectedNodeId) ?? new Set()) : null;
  const panelNodes: GraphPanelNode[] = nodes.map((node) => ({
    ...node,
    degree: degrees.get(node.id) ?? 0,
    neighbors: Array.from(adjacency.get(node.id) ?? [])
      .map((id) => nodeMap.get(id))
      .filter((value): value is AgentKnowledgeGraphNode => Boolean(value))
      .sort((left, right) => left.label.localeCompare(right.label))
  }));

  const displayNodes = panelNodes.map((node) => {
    const isSelected = node.id === selectedNodeId;
    const isNeighbor = selectedNeighbors?.has(node.id) ?? false;
    const isMatch = matchedIds.has(node.id);
    const size = Math.min(60, 28 + node.degree * 3);

    return {
      id: node.id,
      data: { category: node.category, degree: node.degree, label: node.label },
      style: {
        fill: isSelected ? GRAPH_THEME.accentSoft : GRAPH_THEME.nodeFill,
        halo: isSelected || isNeighbor,
        haloFill: isSelected ? GRAPH_THEME.accentSoft : "transparent",
        labelFill: GRAPH_THEME.label,
        labelFontSize: isSelected ? 13 : 12,
        labelFontWeight: isSelected ? 600 : 500,
        labelMaxWidth: 160,
        labelPlacement: "bottom" as const,
        labelText: node.label,
        lineWidth: isSelected ? 2.8 : isNeighbor ? 2 : 1.2,
        opacity: normalizedQuery.length > 0 && !isMatch ? 0.3 : isSelected || isNeighbor ? 1 : 0.92,
        size: isSelected ? size + 6 : size,
        stroke: isSelected
          ? GRAPH_THEME.accent
          : isNeighbor
            ? GRAPH_THEME.edgeHighlight
            : GRAPH_THEME.nodeStroke
      }
    };
  });

  const displayEdges = links.map((link) => {
    const isSelectedEdge =
      selectedNodeId !== null && (link.source === selectedNodeId || link.target === selectedNodeId);
    const isMatchEdge =
      normalizedQuery.length === 0 || matchedIds.has(link.source) || matchedIds.has(link.target);

    return {
      id: `${link.source}::${link.target}`,
      source: link.source,
      target: link.target,
      style: {
        endArrow: true,
        lineDash: isSelectedEdge ? undefined : [5, 4],
        lineWidth: isSelectedEdge ? 2.2 : 1.2,
        opacity: isMatchEdge ? (isSelectedEdge ? 0.95 : 0.55) : 0.15,
        stroke: isSelectedEdge ? GRAPH_THEME.edgeHighlight : GRAPH_THEME.edge
      }
    };
  });

  return {
    filteredNodeCount: matchedIds.size,
    options: {
      animation: true,
      autoResize: true,
      behaviors: ["drag-canvas", "zoom-canvas", "drag-element"],
      data: { edges: displayEdges, nodes: displayNodes },
      edge: { style: { stroke: GRAPH_THEME.edge } },
      layout: {
        collide: {
          radius: (datum: { data?: { degree?: number } }) => 28 + (datum.data?.degree ?? 0) * 2,
          strength: 0.9
        },
        linkDistance: 180,
        manyBody: { strength: -260 },
        type: "d3-force"
      },
      node: {
        style: {
          fill: GRAPH_THEME.nodeFill,
          labelFill: GRAPH_THEME.nodeText,
          lineWidth: 1.2,
          stroke: GRAPH_THEME.nodeStroke
        }
      },
      padding: 24
    },
    selectedNode:
      panelNodes.find((node) => node.id === selectedNodeId) ??
      (normalizedQuery.length > 0
        ? (panelNodes.find((node) => matchedIds.has(node.id)) ?? null)
        : null)
  };
}

function LoadingState({ text }: { text: string }) {
  return (
    <Card
      style={{
        height: "100%",
        minHeight: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center"
      }}
      bodyStyle={{ width: "100%", textAlign: "center" }}
    >
      <Spin tip={text} />
    </Card>
  );
}

function EmptyState({
  hint,
  onUpload,
  uploading
}: {
  hint: string;
  onUpload: KnowledgeGraphPanelProps["onUpload"];
  uploading: boolean;
}) {
  return (
    <Card
      style={{
        height: "100%",
        minHeight: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center"
      }}
      bodyStyle={{ width: "100%" }}
    >
      <Empty image={<IconTreeTriangleDown size="extra-large" />} description={hint}>
        <Button loading={uploading} onClick={onUpload}>
          上传文档
        </Button>
      </Empty>
    </Card>
  );
}

function GraphCanvasSection(props: {
  graphInstance: Graph | null;
  graphState: GraphState;
  linksCount: number;
  nodesCount: number;
  onInit: (graph: Graph) => void;
  onQueryChange: (value: string) => void;
  onReset: () => void;
  query: string;
  summary: string;
}) {
  const { graphState, linksCount, nodesCount, onInit, onQueryChange, onReset, query, summary } =
    props;

  return (
    <Card
      style={{ minHeight: 0, height: "100%", display: "flex", flexDirection: "column" }}
      bodyStyle={{ padding: 0, flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}
    >
      <div style={{ padding: "12px 16px", borderBottom: "1px solid var(--semi-color-border)" }}>
        <Row type="flex" align="middle" justify="space-between" gutter={12}>
          <Col xs={24} lg={12}>
            <Title heading={6} style={{ margin: 0 }}>
              知识图谱
            </Title>
            <Text type="tertiary" size="small" style={{ display: "block", marginTop: 4 }}>
              {`根据文档之间的引用关系，当前共显示 ${nodesCount} 个节点和 ${linksCount} 条关系。`}
            </Text>
          </Col>
          <Col xs={24} lg={12}>
            <Space style={{ width: "100%", justifyContent: "flex-end" }} wrap>
              <Input
                prefix={<IconSearch />}
                value={query}
                onChange={onQueryChange}
                placeholder="按标题、路径或分类搜索节点"
                style={{ minWidth: 220, flex: 1 }}
              />
              <Button icon={<IconRefresh />} theme="light" onClick={onReset}>
                重置视图
              </Button>
            </Space>
          </Col>
        </Row>
      </div>

      <Space
        spacing="tight"
        style={{
          padding: "8px 16px",
          borderBottom: "1px solid var(--semi-color-border)",
          fontSize: 12
        }}
      >
        <Text strong>节点 {summary}</Text>
        <Text type="tertiary">|</Text>
        <Text type="tertiary">引用关系 {linksCount}</Text>
        <Text type="tertiary">|</Text>
        <Text type="tertiary">点击节点可查看关联文档</Text>
      </Space>

      <div style={{ position: "relative", flex: 1, minHeight: 0, background: GRAPH_THEME.canvas }}>
        <Graphin
          style={{ width: "100%", height: "100%", background: GRAPH_THEME.canvas }}
          options={graphState.options}
          onInit={onInit}
          onDestroy={() => undefined}
        />
      </div>
    </Card>
  );
}

function GraphDetailsPanel(props: {
  onNeighborSelect: (id: string | null) => void;
  selectedNode: GraphPanelNode | null;
}) {
  const { onNeighborSelect, selectedNode } = props;

  return (
    <Card
      style={{ minHeight: 0, height: "100%", display: "flex", flexDirection: "column" }}
      bodyStyle={{ padding: 0, flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}
    >
      <div style={{ padding: "12px 16px", borderBottom: "1px solid var(--semi-color-border)" }}>
        <Title heading={6} style={{ margin: 0 }}>
          节点详情
        </Title>
        <Text type="tertiary" size="small" style={{ display: "block", marginTop: 4 }}>
          {selectedNode
            ? "正在查看当前选中的文档节点。"
            : "请在图谱中选择一个节点，查看其关联详情。"}
        </Text>
      </div>

      {selectedNode ? (
        <div style={{ flex: 1, overflowY: "auto", padding: 16 }}>
          <div
            style={{
              background: "var(--semi-color-fill-0)",
              borderRadius: 12,
              padding: 12
            }}
          >
            <Space style={{ width: "100%", justifyContent: "space-between" }} align="start">
              <div style={{ minWidth: 0 }}>
                <Text strong ellipsis={{ showTooltip: true }} style={{ display: "block" }}>
                  {selectedNode.label}
                </Text>
                <Text
                  type="tertiary"
                  size="small"
                  code
                  style={{ display: "block", marginTop: 4, wordBreak: "break-all" }}
                >
                  {selectedNode.id}
                </Text>
              </div>
              <Tag color="blue" size="small">
                {selectedNode.category}
              </Tag>
            </Space>
          </div>

          <Row gutter={12} style={{ marginTop: 16 }}>
            <Col span={12}>
              <Card bodyStyle={{ padding: 12 }}>
                <Text type="tertiary" size="small">
                  连接度
                </Text>
                <Title heading={4} style={{ margin: "4px 0 0" }}>
                  {selectedNode.degree}
                </Title>
              </Card>
            </Col>
            <Col span={12}>
              <Card bodyStyle={{ padding: 12 }}>
                <Text type="tertiary" size="small">
                  关联数量
                </Text>
                <Title heading={4} style={{ margin: "4px 0 0" }}>
                  {selectedNode.neighbors.length}
                </Title>
              </Card>
            </Col>
          </Row>

          <div style={{ marginTop: 16 }}>
            <Space style={{ width: "100%", justifyContent: "space-between", marginBottom: 8 }}>
              <Text strong>关联文档</Text>
              <Button
                icon={<IconClear />}
                theme="borderless"
                type="tertiary"
                size="small"
                onClick={() => onNeighborSelect(null)}
              >
                清除
              </Button>
            </Space>
            {selectedNode.neighbors.length > 0 ? (
              <List
                dataSource={selectedNode.neighbors}
                renderItem={(neighbor) => (
                  <List.Item
                    onClick={() => onNeighborSelect(neighbor.id)}
                    style={{ cursor: "pointer", borderRadius: 8, marginBottom: 4 }}
                    main={
                      <Space vertical align="start" spacing={2} style={{ minWidth: 0 }}>
                        <Text strong ellipsis={{ showTooltip: true }}>
                          {neighbor.label}
                        </Text>
                        <Text type="tertiary" size="small" code ellipsis={{ showTooltip: true }}>
                          {neighbor.id}
                        </Text>
                      </Space>
                    }
                    extra={
                      <Text type="tertiary" size="small">
                        {neighbor.category}
                      </Text>
                    }
                  />
                )}
              />
            ) : (
              <Empty description="当前文档暂无关联链接。" style={{ padding: "16px 0" }} />
            )}
          </div>

          <Card style={{ marginTop: 16, borderStyle: "dashed" }} bodyStyle={{ padding: 12 }}>
            <Space spacing="tight">
              <IconBolt style={{ color: "var(--semi-color-primary)" }} />
              <Text strong>使用提示</Text>
            </Space>
            <Paragraph type="tertiary" size="small" style={{ margin: "8px 0 0" }}>
              在 Markdown 文档之间补充链接，可以让知识图谱更密集、更有用。
            </Paragraph>
          </Card>
        </div>
      ) : (
        <Empty
          style={{ margin: "auto", padding: 24 }}
          image={<IconTreeTriangleDown size="large" />}
          title="请选择节点"
          description="在图中点击一个文档节点，可查看分类、连接度和关联文档。"
        />
      )}
    </Card>
  );
}

export function KnowledgeGraphPanel({
  links,
  loading,
  nodes,
  onUpload,
  uploading
}: KnowledgeGraphPanelProps) {
  const [query, setQuery] = useState("");
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [graphInstance, setGraphInstance] = useState<Graph | null>(null);
  const graphState = useMemo(
    () => buildGraphState({ links, nodes, query, selectedNodeId }),
    [links, nodes, query, selectedNodeId]
  );

  useEffect(() => {
    if (graphInstance) {
      void graphInstance.fitView();
    }
  }, [graphInstance, graphState.options]);

  useEffect(() => {
    if (graphInstance && selectedNodeId) {
      void graphInstance.focusElement(selectedNodeId, { duration: 300 });
    }
  }, [graphInstance, selectedNodeId]);

  if (loading) {
    return <LoadingState text="加载知识库中…" />;
  }

  if (nodes.length === 0) {
    return (
      <EmptyState
        hint="暂无知识文档。点击「上传文档」导入 PDF、Word 等，或在工作区 knowledge/ 添加 Markdown。"
        onUpload={onUpload}
        uploading={uploading}
      />
    );
  }

  return (
    <Row gutter={16} style={{ height: "100%", minHeight: 0, flex: 1 }}>
      <Col span={18} xs={24} lg={18} style={{ minHeight: 0, height: "100%" }}>
        <GraphCanvasSection
          graphInstance={graphInstance}
          graphState={graphState}
          linksCount={links.length}
          nodesCount={nodes.length}
          onInit={(graph) => {
            setGraphInstance(graph);
            graph.on(NodeEvent.CLICK, (event) => {
              const targetId = (event as GraphNodeClickEvent).target?.id;
              if (targetId !== undefined && targetId !== null) {
                setSelectedNodeId(String(targetId));
              }
            });
            graph.on(CanvasEvent.CLICK, () => setSelectedNodeId(null));
          }}
          onQueryChange={setQuery}
          onReset={() => {
            setSelectedNodeId(null);
            void graphInstance?.fitView();
          }}
          query={query}
          summary={`${graphState.filteredNodeCount}/${nodes.length}`}
        />
      </Col>
      <Col span={6} xs={24} lg={6} style={{ minHeight: 0, height: "100%" }}>
        <GraphDetailsPanel
          onNeighborSelect={setSelectedNodeId}
          selectedNode={graphState.selectedNode}
        />
      </Col>
    </Row>
  );
}
