import { FeLogLevel, TauriEvent } from "@/enums";
import type { FeLogPayload } from "@/types/tauri-payloads";

import { isTauriRuntime } from "./invoke";
import { tauriEmit } from "@/utils/tauri-event";

async function writeFeLog(
  eventName: TauriEvent.FeLog | TauriEvent.FeLogReq,
  level: FeLogLevel,
  msg: string,
  consoleOutput: boolean
) {
  if (consoleOutput) {
    console.log(`[${level.toUpperCase()}] ${msg}`);
  }

  if (!isTauriRuntime()) {
    return;
  }

  const payload: FeLogPayload = { level, msg };
  await tauriEmit(eventName, payload);
}

export async function log(level: FeLogLevel, msg: string, consoleOutput: boolean = false) {
  return writeFeLog(TauriEvent.FeLog, level, msg, consoleOutput);
}

export async function log_req(level: FeLogLevel, msg: string, consoleOutput: boolean = false) {
  return writeFeLog(TauriEvent.FeLogReq, level, msg, consoleOutput);
}
