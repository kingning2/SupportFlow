"use client";

import { Skills as SharedSkills } from "@supportflow/ui/agent-console/views/skills";
import { useAgentConsoleState } from "@supportflow/ui/agent-console/hooks/use-agent-console-state";

export function Skills() {
  const { state, setState } = useAgentConsoleState();

  return (
    <div className="min-h-0 flex-1 overflow-y-auto p-6">
      <SharedSkills state={state} onRefresh={setState} />
    </div>
  );
}
