#!/usr/bin/env node
import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(import.meta.url), "..", "..", "apps/full/src");

function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) {
      walk(p, out);
    } else if (/\.tsx?$/.test(name)) {
      out.push(p);
    }
  }
  return out;
}

const rules = [
  ['from "@/components/ui/', 'from "@supportflow/ui/'],
  ['from "@/cmd/', 'from "@supportflow/shared/tauri-bridge/cmd/'],
  ['from "@/enums"', 'from "@supportflow/shared/tauri-bridge/enums"'],
  ['from "@/enums/', 'from "@supportflow/shared/tauri-bridge/enums/'],
  ['from "@/generated/contracts"', 'from "@supportflow/shared/contracts"'],
  ['from "@/lib/utils"', 'from "@supportflow/shared"'],
  ['from "@/utils/tauri-event"', 'from "@supportflow/shared/tauri-bridge/tauri-event"'],
  ['from "@/utils/cache"', 'from "@supportflow/shared/tauri-bridge/cache"'],
  ['from "@/store/', 'from "@supportflow/shared/desktop-shell/store/'],
  ['from "@/providers/', 'from "@supportflow/shared/desktop-shell/providers/'],
  ['from "@/guards/global/', 'from "@supportflow/shared/desktop-shell/guards/global/'],
  ['from "@/events/', 'from "@supportflow/shared/desktop-shell/events/'],
  ['from "@/config/app-config"', 'from "@supportflow/shared/desktop-shell/config/app-config"'],
  ['from "@/config/i18n"', 'from "@supportflow/shared/desktop-shell/config/i18n"'],
  [
    'from "@/hooks/useIsomorphicLayoutEffect"',
    'from "@supportflow/shared/desktop-shell/useIsomorphicLayoutEffect"'
  ],
  ['from "@/components/agent-console"', 'from "@supportflow/ui/agent-console"'],
  ['from "@/types/tauri-payloads"', 'from "@supportflow/shared/contracts/tauri-payloads"'],
  ['from "@/cmd"', 'from "@supportflow/shared/tauri-bridge/cmd"']
];

for (const f of walk(root)) {
  let s = readFileSync(f, "utf8");
  let changed = false;
  for (const [from, to] of rules) {
    if (s.includes(from)) {
      s = s.split(from).join(to);
      changed = true;
    }
  }
  if (changed) writeFileSync(f, s);
}
console.log("apps/full imports updated");
