"use client";

import { Skills as SharedSkills } from "@supportflow/ui/agent-console/views/skills";
import { useAgentConsoleState } from "@supportflow/ui/agent-console/hooks/use-agent-console-state";

export function Skills() {
  const { state, setState } = useAgentConsoleState();

  return <SharedSkills state={state} onRefresh={setState} />;
}
