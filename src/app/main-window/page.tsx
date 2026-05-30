"use client";

import { AgentConsoleApp } from "@/components/agent-console";

/** SupportFlow 控制台（AI Elements + Tauri Agent IPC） */
export default function MainWindowPage() {
  return (
    <div className="-m-3 flex min-h-0 flex-1 flex-col overflow-hidden">
      <AgentConsoleApp />
    </div>
  );
}
