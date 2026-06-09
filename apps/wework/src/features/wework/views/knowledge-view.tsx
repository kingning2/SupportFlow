"use client";

import { useCallback, useEffect, useState } from "react";
import { BookOpen, FolderTree, Network, Trash2, Upload } from "lucide-react";

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
import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";

import { KnowledgeGraphPanel } from "@supportflow/ui/agent-console/views/knowledge-graph-panel";

type KnowledgeTab = "docs" | "graph";

export function KnowledgeView() {
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
          msg += ` / ${"记忆索引已更新"}`;
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
      setStatusMessage(`${"导入失败"}: ${detail}`);
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

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden bg-[var(--main-window-bg)]">
      <div className="border-border/70 bg-card/88 shrink-0 border-b px-6 py-4 backdrop-blur">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="min-w-0">
            <h2 className="text-foreground text-base font-semibold">{"知识库"}</h2>
            <p className="text-muted-foreground mt-1 text-sm">{"浏览和探索你的知识库"}</p>
            {statusMessage ? (
              <p
                className={cn(
                  "mt-2 text-xs whitespace-pre-wrap",
                  statusTone === "success" && "text-success",
                  statusTone === "error" && "text-destructive",
                  statusTone === "info" && "text-muted-foreground"
                )}
              >
                {statusMessage}
              </p>
            ) : null}
          </div>
          <div className="flex items-center gap-2">
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={uploading}
              onClick={handleUploadClick}
            >
              <Upload className="mr-1.5 size-3.5" />
              {uploading ? "正在导入…" : "上传文档"}
            </Button>
            <div className="bg-muted flex items-center rounded-lg p-0.5">
              <Button
                type="button"
                size="sm"
                className={cn("h-8 px-3 text-xs", tab === "docs" && "bg-background")}
                variant={tab === "docs" ? "default" : "ghost"}
                onClick={() => setTab("docs")}
              >
                <FolderTree className="mr-1.5 size-3.5" />
                {"文档"}
              </Button>
              <Button
                type="button"
                size="sm"
                className={cn("h-8 px-3 text-xs", tab === "graph" && "bg-background")}
                variant={tab === "graph" ? "default" : "ghost"}
                onClick={() => setTab("graph")}
              >
                <Network className="mr-1.5 size-3.5" />
                {"图谱"}
              </Button>
            </div>
          </div>
        </div>
      </div>

      <div className="flex min-h-0 flex-1 overflow-hidden p-6">
        {tab === "docs" ? (
          <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[300px_1fr]">
            <div className="bg-card border-border flex min-h-0 flex-col overflow-hidden rounded-2xl border shadow-sm">
              <div className="border-border text-muted-foreground border-b px-4 py-3 text-xs font-medium">
                {"文档"} ({files.length})
              </div>
              <div className="min-h-0 flex-1 overflow-y-auto p-2">
                {loading ? (
                  <p className="text-muted-foreground px-2 py-4 text-sm">{"加载知识库中…"}</p>
                ) : files.length === 0 ? (
                  <div className="space-y-3 px-2 py-4">
                    <p className="text-muted-foreground text-sm">
                      {
                        "暂无知识文档。点击「上传文档」导入 PDF、Word 等，或在工作区 knowledge/ 添加 Markdown。"
                      }
                    </p>
                    <Button
                      type="button"
                      size="sm"
                      variant="secondary"
                      disabled={uploading}
                      onClick={handleUploadClick}
                    >
                      <Upload className="mr-1.5 size-3.5" />
                      {"上传文档"}
                    </Button>
                  </div>
                ) : (
                  files.map((file) => (
                    <div
                      key={file.path}
                      className={cn(
                        "border-border/60 hover:border-primary/35 mb-2 rounded-xl border transition-colors",
                        activePath === file.path && "border-primary/60 bg-primary/6"
                      )}
                    >
                      <button
                        type="button"
                        className="flex w-full flex-col px-3 py-3 text-left"
                        onClick={() => void openFile(file.path)}
                      >
                        <span className="text-foreground text-sm font-medium">{file.title}</span>
                        <span className="text-muted-foreground mt-1 font-mono text-xs">
                          {file.path}
                        </span>
                      </button>
                      <div className="border-border/60 flex items-center justify-end border-t px-3 py-2">
                        <Button
                          type="button"
                          size="sm"
                          variant="ghost"
                          disabled={loading}
                          onClick={() => void handleRemove(file.path)}
                        >
                          <Trash2 className="mr-1.5 size-3.5" />
                          {"删除"}
                        </Button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>

            <div className="bg-card border-border flex min-h-0 flex-col overflow-hidden rounded-2xl border shadow-sm">
              {activePath ? (
                <>
                  <div className="border-border text-muted-foreground border-b px-4 py-3 font-mono text-xs">
                    {activePath}
                  </div>
                  <pre className="text-foreground flex-1 overflow-auto p-4 text-sm whitespace-pre-wrap">
                    {content || "从左侧选择文档查看"}
                  </pre>
                </>
              ) : (
                <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center">
                  <BookOpen className="text-primary size-8" />
                  <p className="text-muted-foreground text-sm">{"从左侧选择文档查看"}</p>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={uploading}
                    onClick={handleUploadClick}
                  >
                    <Upload className="mr-1.5 size-3.5" />
                    {"上传文档"}
                  </Button>
                </div>
              )}
            </div>
          </div>
        ) : (
          <KnowledgeGraphPanel
            nodes={graphNodes}
            links={graphLinks}
            loading={graphLoading}
            uploading={uploading}
            onUpload={handleUploadClick}
          />
        )}
      </div>
    </div>
  );
}
