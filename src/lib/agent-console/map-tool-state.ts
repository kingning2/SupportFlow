import type { DynamicToolUIPart } from "ai";

import type { ToolStepStatus } from "@/types/agent-chat";

/** Map CowAgent tool step status → AI Elements `ToolUIPart` state. */
export function mapToolStepState(status: ToolStepStatus): DynamicToolUIPart["state"] {
  switch (status) {
    case "running":
      return "input-available";
    case "success":
      return "output-available";
    case "error":
      return "output-error";
  }
}
