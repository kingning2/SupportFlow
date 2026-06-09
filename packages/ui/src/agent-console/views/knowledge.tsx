"use client";

import { useCallback, useEffect, useState } from "react";
import { BookOpen, FolderTree, Network, Upload } from "lucide-react";

import {
  getAgentKnowledgeGraph,
  listAgentKnowledge,
  pickAndUploadKnowledge,
  readAgentKnowledge,
  type AgentKnowledgeFile,
  type AgentKnowledgeGraphLink,
  type AgentKnowledgeGraphNode
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";

import { ViewShell } from "../shared/console-brand";
import { KnowledgeGraphPanel } from "./knowledge-graph-panel";

type KnowledgeTab = "docs" | "graph";

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

  // Opens the system file dialog from Rust side. Rust reads the files directly and performs ingest.
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

  return (
    <ViewShell title={"知识库"} description={"浏览和探索你的知识库"}>
      <div className="mx-auto flex h-full w-full max-w-[1600px] flex-col gap-4">
        <div className="flex flex-col justify-between gap-3 sm:flex-row sm:items-center">
          <div className="min-h-5 flex-1">
            {statusMessage ? (
              <p
                className={cn(
                  "text-xs whitespace-pre-wrap",
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
              className="h-8"
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

        {tab === "docs" ? (
          <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[280px_1fr]">
            <div className="bg-card border-border overflow-hidden rounded-xl border">
              <div className="border-border text-muted-foreground border-b px-3 py-2 text-xs font-medium">
                {"文档"} ({files.length})
              </div>
              <div className="max-h-[calc(100vh-240px)] overflow-y-auto p-2">
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
                    <button
                      key={file.path}
                      type="button"
                      className={cn(
                        "hover:bg-accent/40 mb-1 flex w-full flex-col rounded-lg px-3 py-2 text-left text-sm transition-colors",
                        activePath === file.path && "bg-accent/70"
                      )}
                      onClick={() => void openFile(file.path)}
                    >
                      <span className="text-foreground font-medium">{file.title}</span>
                      <span className="text-muted-foreground font-mono text-xs">{file.path}</span>
                    </button>
                  ))
                )}
              </div>
            </div>

            <div className="bg-card border-border flex min-h-[320px] flex-col overflow-hidden rounded-xl border">
              {activePath ? (
                <>
                  <div className="border-border text-muted-foreground border-b px-4 py-2 font-mono text-xs">
                    {activePath}
                  </div>
                  <pre className="text-foreground max-h-[calc(100vh-272px)] flex-1 overflow-y-auto p-4 text-sm whitespace-pre-wrap">
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
    </ViewShell>
  );
}
