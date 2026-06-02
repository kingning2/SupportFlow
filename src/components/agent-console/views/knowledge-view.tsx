"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { BookOpen, FolderTree, Network, Upload } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  getAgentKnowledgeGraph,
  listAgentKnowledge,
  readAgentKnowledge,
  uploadAgentKnowledge,
  type AgentKnowledgeFile,
  type AgentKnowledgeGraphLink,
  type AgentKnowledgeGraphNode
} from "@/cmd/agent";
import { ViewShell } from "@/components/agent-console/shared/console-brand";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type KnowledgeTab = "docs" | "graph";

const KNOWLEDGE_UPLOAD_ACCEPT =
  ".pdf,.docx,.txt,.md,.markdown,.rst,.csv,.tsv,.log,.json,.xml,.html,.htm,.xls,.xlsx,.ppt,.pptx";

async function filesToUploadPayload(files: FileList) {
  const out: { filename: string; data: number[] }[] = [];
  for (const file of Array.from(files)) {
    const buf = await file.arrayBuffer();
    out.push({
      filename: file.name,
      data: Array.from(new Uint8Array(buf))
    });
  }
  return out;
}

export function KnowledgeView() {
  const { t } = useTranslation("console");
  const fileInputRef = useRef<HTMLInputElement>(null);
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

  const handleUploadClick = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const picked = event.target.files;
    event.target.value = "";
    if (!picked?.length) return;

    setUploading(true);
    setStatusMessage(t("knowledge_uploading"));
    setStatusTone("info");

    try {
      const payload = await filesToUploadPayload(picked);
      const result = await uploadAgentKnowledge(payload, "uploads");

      if (result.count > 0) {
        let msg = t("knowledge_upload_success", { count: result.count });
        if (result.memorySynced) {
          msg += ` · ${t("knowledge_upload_memory_synced")}`;
        }
        setStatusMessage(msg);
        setStatusTone("success");
        await loadFiles();
        const first = result.results[0]?.path;
        if (first) {
          await openFile(first);
        }
      } else {
        setStatusMessage(t("knowledge_upload_failed"));
        setStatusTone("error");
      }

      if (result.errors.length > 0) {
        const partial = t("knowledge_upload_partial", { count: result.errors.length });
        const detail = result.errors.map((e) => `${e.file}: ${e.message}`).join("\n");
        setStatusMessage((prev) =>
          prev ? `${prev}\n${partial}\n${detail}` : `${partial}\n${detail}`
        );
        setStatusTone(result.count > 0 ? "success" : "error");
      }
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      setStatusMessage(`${t("knowledge_upload_failed")}: ${detail}`);
      setStatusTone("error");
    } finally {
      setUploading(false);
    }
  };

  return (
    <ViewShell title={t("knowledge_title")} description={t("knowledge_desc")}>
      <div className="mx-auto flex h-full w-full max-w-[1600px] flex-col gap-4">
        <input
          ref={fileInputRef}
          type="file"
          multiple
          accept={KNOWLEDGE_UPLOAD_ACCEPT}
          className="hidden"
          onChange={(e) => void handleFileChange(e)}
        />

        <div className="flex flex-col justify-between gap-3 sm:flex-row sm:items-center">
          <div className="min-h-5 flex-1">
            {statusMessage ? (
              <p
                className={cn(
                  "text-xs whitespace-pre-wrap",
                  statusTone === "success" && "text-emerald-600 dark:text-emerald-400",
                  statusTone === "error" && "text-red-600 dark:text-red-400",
                  statusTone === "info" && "text-slate-500"
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
              {uploading ? t("knowledge_uploading") : t("knowledge_upload_btn")}
            </Button>
            <div className="flex items-center rounded-lg bg-slate-100 p-0.5 dark:bg-white/10">
              <Button
                type="button"
                size="sm"
                className={cn("h-8 px-3 text-xs", tab === "docs" && "bg-white dark:bg-[#1A1A1A]")}
                variant={tab === "docs" ? "default" : "ghost"}
                onClick={() => setTab("docs")}
              >
                <FolderTree className="mr-1.5 size-3.5" />
                {t("knowledge_tab_docs")}
              </Button>
              <Button
                type="button"
                size="sm"
                className={cn("h-8 px-3 text-xs", tab === "graph" && "bg-white dark:bg-[#1A1A1A]")}
                variant={tab === "graph" ? "default" : "ghost"}
                onClick={() => setTab("graph")}
              >
                <Network className="mr-1.5 size-3.5" />
                {t("knowledge_tab_graph")}
              </Button>
            </div>
          </div>
        </div>

        {tab === "docs" ? (
          <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[280px_1fr]">
            <div className="overflow-hidden rounded-xl border border-slate-200 dark:border-white/10">
              <div className="border-b border-slate-200 px-3 py-2 text-xs font-medium text-slate-500 dark:border-white/10">
                {t("knowledge_tab_docs")} ({files.length})
              </div>
              <div className="max-h-[calc(100vh-240px)] overflow-y-auto p-2">
                {loading ? (
                  <p className="px-2 py-4 text-sm text-slate-400">{t("knowledge_loading_desc")}</p>
                ) : files.length === 0 ? (
                  <div className="space-y-3 px-2 py-4">
                    <p className="text-sm text-slate-400">{t("knowledge_empty_hint")}</p>
                    <Button
                      type="button"
                      size="sm"
                      variant="secondary"
                      disabled={uploading}
                      onClick={handleUploadClick}
                    >
                      <Upload className="mr-1.5 size-3.5" />
                      {t("knowledge_upload_btn")}
                    </Button>
                  </div>
                ) : (
                  files.map((file) => (
                    <button
                      key={file.path}
                      type="button"
                      className={cn(
                        "mb-1 flex w-full flex-col rounded-lg px-3 py-2 text-left text-sm transition-colors hover:bg-slate-100 dark:hover:bg-white/5",
                        activePath === file.path && "bg-slate-200/80 dark:bg-white/10"
                      )}
                      onClick={() => void openFile(file.path)}
                    >
                      <span className="font-medium text-slate-700 dark:text-slate-200">
                        {file.title}
                      </span>
                      <span className="font-mono text-xs text-slate-400">{file.path}</span>
                    </button>
                  ))
                )}
              </div>
            </div>

            <div className="flex min-h-[320px] flex-col overflow-hidden rounded-xl border border-slate-200 dark:border-white/10">
              {activePath ? (
                <>
                  <div className="border-b border-slate-200 px-4 py-2 font-mono text-xs text-slate-500 dark:border-white/10">
                    {activePath}
                  </div>
                  <pre className="max-h-[calc(100vh-272px)] flex-1 overflow-y-auto p-4 text-sm whitespace-pre-wrap text-slate-700 dark:text-slate-200">
                    {content || t("knowledge_select_hint")}
                  </pre>
                </>
              ) : (
                <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center">
                  <BookOpen className="size-8 text-emerald-400" />
                  <p className="text-sm text-slate-500">{t("knowledge_select_hint")}</p>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={uploading}
                    onClick={handleUploadClick}
                  >
                    <Upload className="mr-1.5 size-3.5" />
                    {t("knowledge_upload_btn")}
                  </Button>
                </div>
              )}
            </div>
          </div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-slate-200 dark:border-white/10">
            {graphLoading ? (
              <p className="p-8 text-center text-sm text-slate-400">
                {t("knowledge_loading_desc")}
              </p>
            ) : graphNodes.length === 0 ? (
              <div className="flex flex-col items-center gap-3 p-8 text-center">
                <p className="text-sm text-slate-400">{t("knowledge_empty_hint")}</p>
                <Button
                  type="button"
                  size="sm"
                  variant="secondary"
                  disabled={uploading}
                  onClick={handleUploadClick}
                >
                  <Upload className="mr-1.5 size-3.5" />
                  {t("knowledge_upload_btn")}
                </Button>
              </div>
            ) : (
              <div className="grid gap-6 p-4 lg:grid-cols-2">
                <section>
                  <h3 className="mb-2 text-sm font-semibold">
                    {t("knowledge_graph_nodes")} ({graphNodes.length})
                  </h3>
                  <ul className="max-h-[50vh] space-y-1 overflow-y-auto text-sm">
                    {graphNodes.map((node) => (
                      <li
                        key={node.id}
                        className="rounded-md border border-slate-100 px-2 py-1.5 dark:border-white/10"
                      >
                        <span className="font-medium">{node.label}</span>
                        <span className="text-muted-foreground ml-2 font-mono text-xs">
                          {node.category}
                        </span>
                      </li>
                    ))}
                  </ul>
                </section>
                <section>
                  <h3 className="mb-2 text-sm font-semibold">
                    {t("knowledge_graph_links")} ({graphLinks.length})
                  </h3>
                  <ul className="max-h-[50vh] space-y-1 overflow-y-auto font-mono text-xs">
                    {graphLinks.map((link) => (
                      <li
                        key={`${link.source}-${link.target}`}
                        className="rounded-md border border-slate-100 px-2 py-1.5 dark:border-white/10"
                      >
                        {link.source} → {link.target}
                      </li>
                    ))}
                  </ul>
                </section>
              </div>
            )}
          </div>
        )}
      </div>
    </ViewShell>
  );
}
