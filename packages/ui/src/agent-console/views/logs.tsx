"use client";

import { useEffect, useMemo, useState } from "react";
import { Button, Card, Checkbox, Space, Spin, Typography } from "@douyinfe/semi-ui-19";
import { IconCopy, IconTerminal } from "@douyinfe/semi-icons";

import {
  readAgentLogs,
  startAgentLogStream,
  stopAgentLogStream
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import { ViewShell } from "../shared/console-brand";
import { TauriEvent } from "@supportflow/shared/tauri-bridge/enums";
import { tauriOn } from "@supportflow/shared/tauri-bridge/tauri-event";

const { Text } = Typography;

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

function levelColor(level: LogLevel | null): string {
  switch (level) {
    case "critical":
      return "var(--semi-color-text-0)";
    case "error":
      return "var(--semi-color-danger)";
    case "warning":
      return "var(--semi-color-warning)";
    case "info":
      return "var(--semi-color-info)";
    case "debug":
      return "var(--semi-color-text-2)";
    default:
      return "var(--semi-color-text-1)";
  }
}

export function Logs() {
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
        if (mounted) setRaw(result.content ?? "");
      } catch {
        if (mounted) setRaw("");
      } finally {
        if (mounted) setLoading(false);
      }
    };

    void load();

    const unlistenLog = tauriOn<AgentLogStreamPayload>(TauriEvent.AgentLogStream, (event) => {
      const payload = event.payload;
      if (!payload) return;
      if (payload.type === "init") {
        setRaw(payload.content ?? "");
        return;
      }
      if (payload.type === "line" && payload.content) {
        setRaw((prev) => (prev ? `${prev}\n${payload.content}`.trim() : (payload.content ?? "")));
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
      if (!row) continue;
      const lv = lineLevel(row) ?? inherited;
      if (lineLevel(row)) inherited = lineLevel(row);
      if (lv && !enabledLevels[lv]) continue;
      filtered.push({ text: row, level: lv });
    }
    return filtered;
  }, [enabledLevels, raw]);

  useEffect(() => {
    const container = document.getElementById("log-output");
    if (!container || !autoScroll) return;
    container.scrollTop = container.scrollHeight;
  }, [autoScroll, lines]);

  const copySelected = async () => {
    const selected = window.getSelection()?.toString().trim() ?? "";
    if (!selected) return;
    try {
      await navigator.clipboard.writeText(selected);
    } catch {
      // noop
    }
  };

  const copyAll = async () => {
    if (!raw.trim()) return;
    try {
      await navigator.clipboard.writeText(raw);
    } catch {
      // noop
    }
  };

  return (
    <ViewShell title="日志" description="实时日志输出 (run.log)">
      <Card
        bodyStyle={{ padding: 0 }}
        title={
          <Space>
            <IconTerminal />
            <Text code>run.log</Text>
          </Space>
        }
        headerExtraContent={
          <Space wrap>
            <Button
              icon={<IconCopy />}
              theme="borderless"
              type="tertiary"
              size="small"
              onClick={() => void copySelected()}
            >
              复制选中
            </Button>
            <Button
              icon={<IconCopy />}
              theme="borderless"
              type="tertiary"
              size="small"
              onClick={() => void copyAll()}
            >
              复制全部
            </Button>
            {LEVEL_ORDER.map((lv) => (
              <Checkbox
                key={lv}
                checked={enabledLevels[lv]}
                onChange={(e) =>
                  setEnabledLevels((prev) => ({ ...prev, [lv]: Boolean(e.target.checked) }))
                }
              >
                <span style={{ color: levelColor(lv), fontSize: 12 }}>{lv.toUpperCase()}</span>
              </Checkbox>
            ))}
            <Text type="tertiary" size="small">
              实时
            </Text>
          </Space>
        }
      >
        <div
          id="log-output"
          style={{
            height: "calc(100vh - 272px)",
            overflowY: "auto",
            padding: 16,
            fontFamily: "monospace",
            fontSize: 12,
            lineHeight: 1.6,
            wordBreak: "break-all",
            whiteSpace: "pre-wrap",
            userSelect: "text"
          }}
          onScroll={(event) => {
            const el = event.currentTarget;
            const gap = el.scrollHeight - el.scrollTop - el.clientHeight;
            setAutoScroll(gap < 24);
          }}
        >
          {loading ? (
            <Spin tip="加载日志中..." />
          ) : lines.length === 0 ? (
            <Text type="tertiary">暂无日志输出</Text>
          ) : (
            lines.map((line, idx) => (
              <div key={`${idx}-${line.text}`} style={{ color: levelColor(line.level) }}>
                {line.text}
              </div>
            ))
          )}
        </div>
      </Card>
    </ViewShell>
  );
}
