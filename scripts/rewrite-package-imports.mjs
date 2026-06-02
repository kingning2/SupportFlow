#!/usr/bin/env node
import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(import.meta.url), "..", "..");

function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) {
      if (name === "node_modules") continue;
      walk(p, out);
    } else if (/\.tsx?$/.test(name)) {
      out.push(p);
    }
  }
  return out;
}

function apply(file, replacers) {
  let s = readFileSync(file, "utf8");
  let changed = false;
  for (const [from, to] of replacers) {
    if (s.includes(from)) {
      s = s.split(from).join(to);
      changed = true;
    }
  }
  if (changed) writeFileSync(file, s);
}

// --- tauri-bridge ---
for (const f of walk(join(root, "packages/tauri-bridge/src"))) {
  apply(f, [
    ['from "@/cmd/invoke"', 'from "./invoke"'],
    ['from "@/cmd"', 'from "./index"'],
    ['from "@/enums/tauri-cmd"', 'from "../enums/tauri-cmd"'],
    ['from "@/enums"', 'from "../enums"'],
    ['from "@/generated/contracts"', 'from "@supportflow/shared/contracts"'],
    ['from "@/types/tauri-payloads"', 'from "@supportflow/shared/contracts/tauri-payloads"'],
    ['from "@/utils/tauri-event"', 'from "../tauri-event"'],
    ['from "@/utils/cache"', 'from "../cache"'],
    ['from ".";', 'from "./invoke";']
  ]);
}
// fix cmd/index
writeFileSync(
  join(root, "packages/tauri-bridge/src/cmd/index.ts"),
  `export { InvokeError, invokeWrapper, isTauriRuntime } from "./invoke";
export { TauriCmd } from "../enums";
`
);

// --- ui ---
for (const f of walk(join(root, "packages/ui/src"))) {
  apply(f, [
    ['from "@/lib/utils"', 'from "@supportflow/shared"'],
    ['from "@/components/ui/', 'from "@supportflow/ui/']
  ]);
}

// --- desktop-shell ---
for (const f of walk(join(root, "packages/desktop-shell/src"))) {
  apply(f, [
    ['from "@/cmd/lang"', 'from "@supportflow/shared/tauri-bridge/cmd/lang"'],
    ['from "@/cmd/session"', 'from "@supportflow/shared/tauri-bridge/cmd/session"'],
    ['from "@/cmd/window"', 'from "@supportflow/shared/tauri-bridge/cmd/window"'],
    ['from "@/enums"', 'from "@supportflow/shared/tauri-bridge/enums"'],
    ['from "@/store/hooks"', 'from "../store/hooks"'],
    ['from "@/store/modules/app"', 'from "../store/modules/app"'],
    ['from "@/store/types"', 'from "../store/types"'],
    ['from "@/store"', 'from "../store"'],
    ['from "@/config/app-config"', 'from "../config/app-config"'],
    ['from "@/config/i18n"', 'from "../config/i18n"'],
    ['from "@/events/cross-webview-sync"', 'from "../events/cross-webview-sync"'],
    ['from "@/utils/tauri-event"', 'from "@supportflow/shared/tauri-bridge/tauri-event"'],
    ['from "@/utils/cache"', 'from "@supportflow/shared/tauri-bridge/cache"'],
    ['from "@/hooks/useIsomorphicLayoutEffect"', 'from "../useIsomorphicLayoutEffect"'],
    ['from "@/providers/', 'from "../providers/']
  ]);
}

// --- agent-console ---
const acPrefix = [
  ['from "@/components/ui/', 'from "@supportflow/ui/'],
  ['from "@/cmd/agent"', 'from "@supportflow/shared/tauri-bridge/cmd/agent"'],
  [
    'from "@/cmd/channel-python-channels"',
    'from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels"'
  ],
  [
    'from "@/enums/agent-stream-chunk-type"',
    'from "@supportflow/shared/tauri-bridge/enums/agent-stream-chunk-type"'
  ],
  ['from "@/enums/dev-channel"', 'from "@supportflow/shared/tauri-bridge/enums/dev-channel"'],
  ['from "@/enums"', 'from "@supportflow/shared/tauri-bridge/enums"'],
  ['from "@/generated/contracts"', 'from "@supportflow/shared/contracts"'],
  ['from "@/lib/utils"', 'from "@supportflow/shared"'],
  ['from "@/utils/tauri-event"', 'from "@supportflow/shared/tauri-bridge/tauri-event"'],
  ['from "@/store/hooks"', 'from "@supportflow/shared/desktop-shell/store/hooks"'],
  ['from "@/store/modules/app"', 'from "@supportflow/shared/desktop-shell/store/modules/app"'],
  ['from "@/types/agent-chat"', 'from "../types/agent-chat"'],
  ['from "@/hooks/use-agent-console-state"', 'from "../hooks/use-agent-console-state"'],
  ['from "@/hooks/use-agent-chat"', 'from "../hooks/use-agent-chat"'],
  ['from "@/lib/agent-console/', 'from "../lib/agent-console/'],
  ['from "@/components/ai-elements/', 'from "../ai-elements/'],
  ['from "@/components/agent-console/constants/', 'from "../constants/'],
  ['from "@/components/agent-console/shared/', 'from "../shared/'],
  ['from "@/components/agent-console/layout/', 'from "../layout/'],
  ['from "@/components/agent-console/chat/', 'from "../chat/'],
  ['from "@/components/agent-console/views/channels/', 'from "../views/channels/'],
  ['from "@/components/agent-console/views/', 'from "../views/'],
  [
    "@/components/agent-console/styles/console.css",
    "@supportflow/ui/agent-console/styles/console.css"
  ]
];
for (const f of walk(join(root, "packages/agent-console/src"))) {
  apply(f, acPrefix);
  // depth fix for nested files - types/agent-chat from views needs ../../types
  let s = readFileSync(f, "utf8");
  const depth = f.split(/[/\\]/).slice(-1)[0].includes(".")
    ? f.replace(/\\/g, "/").split("/src/")[1].split("/").length - 1
    : 0;
  // fix relative types import in nested folders
  if (f.includes("/views/") || f.includes("/chat/") || f.includes("/layout/")) {
    s = s.replace('from "../types/agent-chat"', 'from "../../types/agent-chat"');
    s = s.replace('from "../hooks/use-agent-chat"', 'from "../../hooks/use-agent-chat"');
    s = s.replace(
      'from "../hooks/use-agent-console-state"',
      'from "../../hooks/use-agent-console-state"'
    );
    s = s.replace('from "../lib/agent-console/', 'from "../../lib/agent-console/');
  }
  if (f.includes("/views/channels/")) {
    s = s.replace('from "../../types/agent-chat"', 'from "../../../types/agent-chat"');
    s = s.replace('from "../../lib/agent-console/', 'from "../../../lib/agent-console/');
    s = s.replace('from "../views/channels/', 'from "./');
  }
  writeFileSync(f, s);
}

// agent-console index
writeFileSync(
  join(root, "packages/agent-console/src/index.ts"),
  `export { AgentConsoleApp } from "./layout/agent-console-app";
import "@supportflow/ui/agent-console/styles/console.css";
`
);

console.log("done");
