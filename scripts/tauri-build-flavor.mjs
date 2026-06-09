#!/usr/bin/env node
/**
 * Tauri release build for a channel flavor (standalone frontend, no other channel bundles).
 * Usage: node scripts/tauri-build-flavor.mjs full|wework|wechat
 */
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const FLAVOR_CONFIG = {
  wework: "src-tauri/tauri.wework.conf.json"
};

const flavor = process.argv[2]?.trim().toLowerCase();
const config = FLAVOR_CONFIG[flavor];

if (!config) {
  console.error(
    "Usage: node scripts/tauri-build-flavor.mjs <flavor>\nFlavors: " +
      Object.keys(FLAVOR_CONFIG).join(", ")
  );
  process.exit(1);
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const env = {
  ...process.env,
  DEV_CHANNEL: flavor,
  NEXT_PUBLIC_DEV_CHANNEL: flavor
};

console.log(`[tauri-build-flavor] flavor=${flavor} config=${config}`);

const result = spawnSync("pnpm", ["run", "tauri", "build", "--config", config], {
  cwd: root,
  env,
  stdio: "inherit",
  shell: true
});

process.exit(result.status ?? 1);
