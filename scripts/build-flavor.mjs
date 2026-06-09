#!/usr/bin/env node
/**
 * Build a channel-specific frontend (excludes other channel apps from output).
 * Usage: node scripts/build-flavor.mjs wework|wechat|full
 */
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const FLAVORS = {
  wework: { cwd: "apps/wework", label: "企微个人号 (wework)" }
};

const flavor = process.argv[2]?.trim().toLowerCase();
const spec = FLAVORS[flavor];

if (!spec) {
  console.error(
    "Usage: node scripts/build-flavor.mjs <flavor>\nFlavors: " + Object.keys(FLAVORS).join(", ")
  );
  process.exit(1);
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

console.log(`[build-flavor] ${spec.label}`);

const appDir = path.join(root, spec.cwd);
const result = spawnSync("pnpm", ["run", "build"], { cwd: appDir, stdio: "inherit", shell: true });
process.exit(result.status ?? 1);
