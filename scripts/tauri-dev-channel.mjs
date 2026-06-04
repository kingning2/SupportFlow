#!/usr/bin/env node
/**
 * Run `tauri dev` with a single supported personal channel preset.
 * Usage: node scripts/tauri-dev-channel.mjs wechat
 */
import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const CHANNEL_IDS = new Set(["wx", "wework"]);

const ALIASES = {
  wechat: "wx",
  personal_wechat: "wx",
  wx: "wx",
  wework: "wework"
};

/** Channel id → standalone Tauri config (separate Next app, isolated bundle). */
const STANDALONE_TAURI_CONFIG = {
  wework: "src-tauri/tauri.wework.conf.json",
  wx: "src-tauri/tauri.wechat.conf.json"
};

function resolveChannel(raw) {
  const key = String(raw ?? "")
    .trim()
    .toLowerCase();
  if (!key) {
    return null;
  }
  if (ALIASES[key]) {
    return ALIASES[key];
  }
  if (CHANNEL_IDS.has(key)) {
    return key;
  }
  return null;
}

const channel = resolveChannel(process.argv[2]);
if (!channel) {
  console.error(
    "Usage: node scripts/tauri-dev-channel.mjs <channel>\n" +
      "Channels: " +
      [...CHANNEL_IDS].join(", ") +
      "\nAliases: wechat→wx"
  );
  process.exit(1);
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const env = {
  ...process.env,
  NEXT_PUBLIC_DEV_CHANNEL: channel,
  DEV_CHANNEL: channel
};

const standaloneConfig = STANDALONE_TAURI_CONFIG[channel];
const tauriArgs = ["run", "tauri", "dev", "--no-watch"];
if (standaloneConfig) {
  tauriArgs.push("--config", standaloneConfig);
}

console.log(
  `[tauri-dev-channel] preset=${channel}` +
    (standaloneConfig ? ` standalone=${standaloneConfig}` : " full-console")
);

const child = spawn("pnpm", tauriArgs, {
  cwd: root,
  env,
  stdio: "inherit",
  shell: true
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});
