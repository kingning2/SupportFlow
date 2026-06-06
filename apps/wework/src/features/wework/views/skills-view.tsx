"use client";

import { SkillsView as SharedSkillsView } from "@supportflow/ui/agent-console/views/skills-view";
import { useAgentConsoleState } from "@supportflow/ui/agent-console/hooks/use-agent-console-state";

export function SkillsView() {
  const { state, setState } = useAgentConsoleState();

  return (
    <div className="min-h-0 flex-1 overflow-y-auto p-6">
      <SharedSkillsView state={state} onRefresh={setState} />
    </div>
  );
}
