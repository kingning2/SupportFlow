"use client";

import { useEffect, useMemo, useState } from "react";
import { Copy, Terminal } from "lucide-react";

import {
  readAgentLogs,
  startAgentLogStream,
  stopAgentLogStream
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { ViewShell } from "../shared/console-brand";
import { Button } from "@supportflow/ui/button";
import { Checkbox } from "@supportflow/ui/checkbox";
import { TauriEvent } from "@supportflow/shared/tauri-bridge/enums";
import { tauriOn } from "@supportflow/shared/tauri-bridge/tauri-event";

type LogLevel = "debug" | "info" | "warning" | "error" | "critical";
type AgentLogStreamPayload = {
  type: "init" | "line" | "error";
  content?: string | null;
  message?: string | null;
};

const LEVEL_ORDER: LogLevel[] = ["debug", "info", "warning", "error", "critical"];

function lineLevel(line: string): LogLevel | null {
  if (/\[CRITICAL\]/.test(line)) return "critical";
  if (/\[ERROR\]/.test(line)) return "error";
  if (/\[WARNING\]|\[WARN\]/.test(line)) return "warning";
  if (/\[INFO\]/.test(line)) return "info";
  if (/\[DEBUG\]/.test(line)) return "debug";
  return null;
}

function levelClass(level: LogLevel | null) {
  switch (level) {
    case "critical":
      return "text-white font-semibold";
    case "error":
      return "text-destructive";
    case "warning":
      return "text-warning";
    case "info":
      return "text-info";
    case "debug":
      return "text-muted-foreground";
    default:
      return "text-foreground/80";
  }
}

export function LogsView() {
  const [raw, setRaw] = useState("");
  const [loading, setLoading] = useState(true);
  const [enabledLevels, setEnabledLevels] = useState<Record<LogLevel, boolean>>({
    debug: true,
    info: true,
    warning: true,
    error: true,
    critical: true
  });
  const [autoScroll, setAutoScroll] = useState(true);

  useEffect(() => {
    let mounted = true;

    const load = async () => {
      try {
        const result = await readAgentLogs({ limit: 500 });
        if (mounted) {
          setRaw(result.content ?? "");
        }
      } catch {
        if (mounted) {
          setRaw("");
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    void load();

    const unlistenLog = tauriOn<AgentLogStreamPayload>(TauriEvent.AgentLogStream, (event) => {
      const payload = event.payload;
      if (!payload) {
        return;
      }
      if (payload.type === "init") {
        setRaw(payload.content ?? "");
        return;
      }
      if (payload.type === "line") {
        if (!payload.content) {
          return;
        }
        setRaw((prev) => {
          if (!prev) {
            return payload.content ?? "";
          }
          return `${prev}\n${payload.content ?? ""}`.trim();
        });
      }
    });

    void startAgentLogStream();

    return () => {
      mounted = false;
      unlistenLog();
      void stopAgentLogStream();
    };
  }, []);

  const lines = useMemo(() => {
    const rows = raw.split("\n");
    const filtered: { text: string; level: LogLevel | null }[] = [];
    let inherited: LogLevel | null = null;

    for (const row of rows) {
      if (!row) {
        continue;
      }
      const lv = lineLevel(row) ?? inherited;
      if (lineLevel(row)) {
        inherited = lineLevel(row);
      }
      if (lv && !enabledLevels[lv]) {
        continue;
      }
      filtered.push({ text: row, level: lv });
    }
    return filtered;
  }, [enabledLevels, raw]);

  useEffect(() => {
    const container = document.getElementById("log-output");
    if (!container || !autoScroll) {
      return;
    }
    container.scrollTop = container.scrollHeight;
  }, [autoScroll, lines]);

  const copySelected = async () => {
    const selected = window.getSelection()?.toString().trim() ?? "";
    if (!selected) {
      return;
    }
    try {
      await navigator.clipboard.writeText(selected);
    } catch {
      // noop
    }
  };

  const copyAll = async () => {
    if (!raw.trim()) {
      return;
    }
    try {
      await navigator.clipboard.writeText(raw);
    } catch {
      // noop
    }
  };

  return (
    <ViewShell title={"日志"} description={"实时日志输出 (run.log)"}>
      <div className="mx-auto h-full w-full max-w-5xl">
        <div className="bg-surface-2 border-border overflow-hidden rounded-xl border shadow-lg">
          <div className="bg-surface-1 border-border flex items-center gap-2 border-b px-4 py-2.5">
            <Terminal className="text-muted-foreground size-3.5" />
            <span className="text-muted-foreground font-mono text-xs">run.log</span>
            <div className="flex-1" />
            <div className="mr-2 flex items-center gap-1">
              <Button
                type="button"
                size="sm"
                variant="ghost"
                className="text-foreground/80 hover:text-foreground h-7 px-2"
                onClick={() => void copySelected()}
              >
                <Copy className="mr-1 size-3.5" />
                {"复制选中"}
              </Button>
              <Button
                type="button"
                size="sm"
                variant="ghost"
                className="text-foreground/80 hover:text-foreground h-7 px-2"
                onClick={() => void copyAll()}
              >
                <Copy className="mr-1 size-3.5" />
                {"复制全部"}
              </Button>
            </div>
            <div className="mr-2 flex items-center gap-3">
              {LEVEL_ORDER.map((lv) => (
                <label key={lv} className="text-foreground/80 flex items-center gap-1 text-xs">
                  <Checkbox
                    checked={enabledLevels[lv]}
                    onChange={(event) =>
                      setEnabledLevels((prev) => ({
                        ...prev,
                        [lv]: Boolean(event.target.checked)
                      }))
                    }
                  />
                  <span className={levelClass(lv)}>{lv.toUpperCase()}</span>
                </label>
              ))}
            </div>
            <span className="text-muted-foreground text-xs">{"实时"}</span>
          </div>
          <div
            id="log-output"
            className="text-foreground/80 overflow-y-auto p-4 font-mono text-xs leading-relaxed break-all whitespace-pre-wrap select-text"
            style={{ height: "calc(100vh - 272px)" }}
            onScroll={(event) => {
              const el = event.currentTarget;
              const gap = el.scrollHeight - el.scrollTop - el.clientHeight;
              setAutoScroll(gap < 24);
            }}
          >
            {loading ? (
              <p className="text-muted-foreground">{"加载日志中..."}</p>
            ) : lines.length === 0 ? (
              <p className="text-muted-foreground">{"暂无日志输出"}</p>
            ) : (
              lines.map((line, idx) => (
                <span key={`${idx}-${line.text}`} className={`${levelClass(line.level)} block`}>
                  {line.text}
                </span>
              ))
            )}
          </div>
        </div>
      </div>
    </ViewShell>
  );
}
