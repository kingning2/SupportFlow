"use client";

import { Graphin } from "@antv/graphin";
import { CanvasEvent, type Graph, NodeEvent } from "@antv/g6";
import { Network, RotateCcw, Search, Sparkles } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type {
  AgentKnowledgeGraphLink,
  AgentKnowledgeGraphNode
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";
import { Input } from "@supportflow/ui/input";

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
    <div className="bg-card border-border flex h-full min-h-0 items-center justify-center rounded-2xl border">
      <p className="text-muted-foreground text-sm">{text}</p>
    </div>
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
    <div className="bg-card border-border flex h-full min-h-0 flex-col items-center justify-center gap-3 rounded-2xl border p-8 text-center">
      <Network className="text-muted-foreground size-10" />
      <p className="text-muted-foreground max-w-md text-sm">{hint}</p>
      <Button type="button" size="sm" variant="secondary" disabled={uploading} onClick={onUpload}>
        Upload
      </Button>
    </div>
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
  t: ReturnType<typeof useTranslation>["t"];
}) {
  const { graphState, linksCount, nodesCount, onInit, onQueryChange, onReset, query, summary, t } =
    props;
  return (
    <section className="bg-card border-border flex min-h-0 flex-col overflow-hidden rounded-2xl border">
      <div className="border-border flex flex-col gap-3 border-b px-4 py-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h3 className="text-foreground text-sm font-semibold">
            {t("knowledge_graph_canvas_title")}
          </h3>
          <p className="text-muted-foreground mt-1 text-xs">
            {t("knowledge_graph_canvas_desc", { links: linksCount, nodes: nodesCount })}
          </p>
        </div>
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
          <div className="relative min-w-[220px] flex-1">
            <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2" />
            <Input
              value={query}
              onChange={(event) => onQueryChange(event.target.value)}
              placeholder={t("knowledge_graph_search_placeholder")}
              className="bg-background border-border pl-9"
            />
          </div>
          <Button type="button" size="sm" variant="outline" onClick={onReset}>
            <RotateCcw className="mr-1.5 size-3.5" />
            {t("knowledge_graph_reset_view")}
          </Button>
        </div>
      </div>

      <div className="border-border bg-muted/25 flex items-center gap-2 border-b px-4 py-2 text-xs">
        <span className="text-foreground font-medium">
          {t("knowledge_graph_nodes")} {summary}
        </span>
        <span className="text-muted-foreground">|</span>
        <span className="text-muted-foreground">
          {t("knowledge_graph_links")} {linksCount}
        </span>
        <span className="text-muted-foreground">|</span>
        <span className="text-muted-foreground">{t("knowledge_graph_interaction_hint")}</span>
      </div>

      <div className="relative flex-1 bg-[hsl(var(--surface-0,var(--background)))]">
        <Graphin
          className="h-full w-full"
          style={{ background: GRAPH_THEME.canvas }}
          options={graphState.options}
          onInit={onInit}
          onDestroy={() => undefined}
        />
      </div>
    </section>
  );
}

function GraphDetailsPanel(props: {
  onNeighborSelect: (id: string | null) => void;
  selectedNode: GraphPanelNode | null;
  t: ReturnType<typeof useTranslation>["t"];
}) {
  const { onNeighborSelect, selectedNode, t } = props;
  return (
    <aside className="bg-card border-border flex min-h-0 flex-col overflow-hidden rounded-2xl border">
      <div className="border-border border-b px-4 py-3">
        <h3 className="text-foreground text-sm font-semibold">
          {t("knowledge_graph_details_title")}
        </h3>
        <p className="text-muted-foreground mt-1 text-xs">
          {selectedNode
            ? t("knowledge_graph_details_selected")
            : t("knowledge_graph_details_empty")}
        </p>
      </div>

      {selectedNode ? (
        <div className="flex flex-1 flex-col overflow-y-auto p-4">
          <div className="bg-muted/50 rounded-xl p-3">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <p className="text-foreground truncate text-sm font-semibold">
                  {selectedNode.label}
                </p>
                <p className="text-muted-foreground mt-1 font-mono text-[11px] break-all">
                  {selectedNode.id}
                </p>
              </div>
              <span className="bg-primary/10 text-primary rounded-full px-2 py-1 text-[11px] font-medium">
                {selectedNode.category}
              </span>
            </div>
          </div>

          <div className="mt-4 grid grid-cols-2 gap-3">
            <div className="bg-background border-border rounded-xl border p-3">
              <p className="text-muted-foreground text-[11px]">{t("knowledge_graph_degree")}</p>
              <p className="text-foreground mt-1 text-lg font-semibold">{selectedNode.degree}</p>
            </div>
            <div className="bg-background border-border rounded-xl border p-3">
              <p className="text-muted-foreground text-[11px]">
                {t("knowledge_graph_neighbor_count")}
              </p>
              <p className="text-foreground mt-1 text-lg font-semibold">
                {selectedNode.neighbors.length}
              </p>
            </div>
          </div>

          <div className="mt-4">
            <div className="mb-2 flex items-center justify-between gap-2">
              <p className="text-foreground text-sm font-medium">
                {t("knowledge_graph_related_nodes")}
              </p>
              <Button
                type="button"
                size="sm"
                variant="ghost"
                onClick={() => onNeighborSelect(null)}
              >
                {t("knowledge_graph_clear_selection")}
              </Button>
            </div>
            <div className="space-y-2">
              {selectedNode.neighbors.length > 0 ? (
                selectedNode.neighbors.map((neighbor) => (
                  <button
                    key={neighbor.id}
                    type="button"
                    className={cn(
                      "bg-background border-border hover:border-primary/40 hover:bg-accent/35 flex w-full items-center justify-between rounded-xl border px-3 py-2 text-left transition-colors"
                    )}
                    onClick={() => onNeighborSelect(neighbor.id)}
                  >
                    <div className="min-w-0">
                      <p className="text-foreground truncate text-sm font-medium">
                        {neighbor.label}
                      </p>
                      <p className="text-muted-foreground mt-0.5 truncate font-mono text-[11px]">
                        {neighbor.id}
                      </p>
                    </div>
                    <span className="text-muted-foreground shrink-0 text-[11px]">
                      {neighbor.category}
                    </span>
                  </button>
                ))
              ) : (
                <div className="bg-muted/35 rounded-xl px-3 py-4 text-center">
                  <p className="text-muted-foreground text-sm">
                    {t("knowledge_graph_no_neighbors")}
                  </p>
                </div>
              )}
            </div>
          </div>

          <div className="mt-4 rounded-xl border border-dashed border-[hsl(var(--border))] bg-[hsl(var(--surface-1,var(--muted)))]/50 p-3">
            <div className="flex items-center gap-2">
              <Sparkles className="text-primary size-4" />
              <p className="text-foreground text-sm font-medium">
                {t("knowledge_graph_tip_title")}
              </p>
            </div>
            <p className="text-muted-foreground mt-2 text-xs leading-5">
              {t("knowledge_graph_tip_body")}
            </p>
          </div>
        </div>
      ) : (
        <div className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
          <Network className="text-muted-foreground size-8" />
          <p className="text-foreground text-sm font-medium">{t("knowledge_graph_pick_node")}</p>
          <p className="text-muted-foreground max-w-[240px] text-xs leading-5">
            {t("knowledge_graph_pick_node_desc")}
          </p>
        </div>
      )}
    </aside>
  );
}

export function KnowledgeGraphPanel({
  links,
  loading,
  nodes,
  onUpload,
  uploading
}: KnowledgeGraphPanelProps) {
  const { t } = useTranslation("console");
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
    return <LoadingState text={t("knowledge_loading_desc")} />;
  }

  if (nodes.length === 0) {
    return (
      <EmptyState hint={t("knowledge_empty_hint")} onUpload={onUpload} uploading={uploading} />
    );
  }

  return (
    <div className="grid h-full min-h-0 flex-1 grid-cols-1 gap-4 lg:grid-cols-[3fr_1fr]">
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
        t={t}
      />
      <GraphDetailsPanel
        onNeighborSelect={setSelectedNodeId}
        selectedNode={graphState.selectedNode}
        t={t}
      />
    </div>
  );
}
