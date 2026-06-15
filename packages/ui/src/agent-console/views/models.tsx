"use client";

import type { AgentConsoleState } from "@supportflow/shared/contracts";

import { ProviderSettings } from "./provider-settings";

interface ModelsProps {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
}

export function Models({ state, onRefresh }: ModelsProps) {
  return <ProviderSettings state={state} onRefresh={onRefresh} showRuntimePanel={false} />;
}
