import { InvokeError } from "./cmd/invoke";

export const LICENSE_LOCKED_PREFIX = "LICENSE_LOCKED";

export function isLicenseLockedMessage(message: string): boolean {
  return message.includes(LICENSE_LOCKED_PREFIX);
}

export function isLicenseLockedError(error: unknown): boolean {
  if (error instanceof InvokeError || error instanceof Error) {
    return isLicenseLockedMessage(error.message);
  }
  return isLicenseLockedMessage(String(error));
}

export function parseLicenseLockReason(error: unknown): string | null {
  const message = error instanceof Error ? error.message : String(error);
  const match = message.match(/LICENSE_LOCKED:\s*(.+)/i);
  if (match?.[1]?.trim()) {
    return match[1].trim();
  }
  return isLicenseLockedMessage(message) ? "license locked" : null;
}

/** 将后端 reason 转为面向用户的简短说明。 */
export function formatLicenseInvalidReason(reason?: string | null): string {
  if (!reason?.trim()) {
    return "订阅未激活或已失效";
  }
  const normalized = reason.trim().toLowerCase();
  if (normalized.includes("expired")) {
    return "订阅已过期";
  }
  if (normalized.includes("missing activation")) {
    return "尚未激活订阅";
  }
  if (normalized.includes("invalid activation")) {
    return "激活码无效";
  }
  return reason.trim();
}
