import { invoke, InvokeArgs, InvokeOptions } from "@tauri-apps/api/core";

import { FeLogLevel, TauriCmd } from "../enums";

export class InvokeError extends Error {
  readonly cmd: TauriCmd;

  constructor(cmd: TauriCmd, cause: unknown) {
    const detail = cause instanceof Error ? cause.message : String(cause);
    super(`${cmd}: ${detail}`);
    this.name = "InvokeError";
    this.cmd = cmd;
    if (cause instanceof Error && cause.stack) {
      this.cause = cause;
    }
  }
}

type InvokeErrorListener = (error: InvokeError) => void;

const invokeErrorListeners = new Set<InvokeErrorListener>();

export function subscribeInvokeErrors(listener: InvokeErrorListener): () => void {
  invokeErrorListeners.add(listener);
  return () => {
    invokeErrorListeners.delete(listener);
  };
}

function notifyInvokeError(error: InvokeError): void {
  for (const listener of invokeErrorListeners) {
    try {
      listener(error);
    } catch {
      // ignore listener failures
    }
  }
}

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function logInvokeFailure(cmd: TauriCmd, error: unknown): Promise<void> {
  const detail = error instanceof Error ? error.message : String(error);
  const msg = `[invoke:${cmd}] ${detail}`;

  if (!isTauriRuntime()) {
    console.error(msg, error);
    return;
  }

  try {
    await invoke(TauriCmd.LogFe, { event: FeLogLevel.Error, msg });
  } catch {
    console.error(msg, error);
  }
}

export const invokeWrapper = async <T>(
  cmd: TauriCmd,
  args?: InvokeArgs,
  options?: InvokeOptions
): Promise<T> => {
  if (!isTauriRuntime()) {
    const err = new InvokeError(cmd, new Error("not running in Tauri"));
    void logInvokeFailure(cmd, err);
    throw err;
  }

  try {
    return await invoke<T>(cmd, args, options);
  } catch (error) {
    void logInvokeFailure(cmd, error);
    const err = new InvokeError(cmd, error);
    notifyInvokeError(err);
    throw err;
  }
};
