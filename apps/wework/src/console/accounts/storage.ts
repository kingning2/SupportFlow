import type { ChannelCatalogEntry } from "@supportflow/shared";
import { LocalCacheKey } from "@supportflow/shared/tauri-bridge/enums";
import {
  weworkDeleteAccount,
  weworkGetActiveAccountId,
  weworkListAccounts,
  weworkSetActiveAccountId,
  weworkUpsertAccount,
  type WeworkSavedAccountDto
} from "@supportflow/shared/tauri-bridge/cmd/wework-accounts";

import type { WeworkAccountConfig, WeworkSavedAccount } from "./types";

function readLegacyLocalAccounts(): WeworkSavedAccount[] {
  if (typeof window === "undefined") {
    return [];
  }
  const raw = localStorage.getItem(LocalCacheKey.WeworkSavedAccounts);
  if (!raw) {
    return [];
  }
  try {
    const parsed = JSON.parse(raw) as WeworkSavedAccount[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

async function migrateLegacyLocalStorage(
  accounts: WeworkSavedAccount[]
): Promise<WeworkSavedAccount[]> {
  if (accounts.length > 0) {
    return accounts;
  }
  const legacy = readLegacyLocalAccounts();
  if (legacy.length === 0) {
    return accounts;
  }
  let next = accounts;
  for (const item of legacy) {
    const { accounts: merged } = await upsertSavedAccount(
      next,
      item.config,
      item.label,
      item.weworkUserId
    );
    next = merged;
  }
  const legacyActive = localStorage.getItem(LocalCacheKey.WeworkActiveAccountId);
  if (legacyActive) {
    await persistActiveAccountId(legacyActive);
  }
  localStorage.removeItem(LocalCacheKey.WeworkSavedAccounts);
  localStorage.removeItem(LocalCacheKey.WeworkActiveAccountId);
  return next;
}

function dtoToAccount(dto: WeworkSavedAccountDto): WeworkSavedAccount {
  return {
    id: dto.id,
    label: dto.label,
    config: {
      wework_exe_path: dto.config.weworkExePath,
      wework_version: dto.config.weworkVersion,
      wework_smart: dto.config.weworkSmart,
      wework_init_wait_seconds: dto.config.weworkInitWaitSeconds
    },
    createdAt: dto.createdAt,
    lastConnectedAt: dto.lastConnectedAt,
    weworkUserId: dto.weworkUserId
  };
}

function accountToDto(account: WeworkSavedAccount): WeworkSavedAccountDto {
  return {
    id: account.id,
    label: account.label,
    config: {
      weworkExePath: account.config.wework_exe_path,
      weworkVersion: account.config.wework_version,
      weworkSmart: account.config.wework_smart,
      weworkInitWaitSeconds: account.config.wework_init_wait_seconds
    },
    createdAt: account.createdAt,
    lastConnectedAt: account.lastConnectedAt,
    weworkUserId: account.weworkUserId
  };
}

export function configFingerprint(config: WeworkAccountConfig): string {
  return (config.wework_exe_path ?? "").trim().toLowerCase();
}

export function defaultAccountLabel(config: WeworkAccountConfig): string {
  const path = (config.wework_exe_path ?? "").replace(/\\/g, "/").trim();
  if (!path) {
    return "";
  }
  const parts = path.split("/").filter(Boolean);
  if (parts.length >= 2) {
    return parts[parts.length - 2] ?? parts[parts.length - 1] ?? "";
  }
  return parts[parts.length - 1] ?? "";
}

export async function loadSavedAccounts(): Promise<WeworkSavedAccount[]> {
  const list = await weworkListAccounts();
  const accounts = list.map(dtoToAccount);
  return migrateLegacyLocalStorage(accounts);
}

export async function loadActiveAccountId(): Promise<string | null> {
  return weworkGetActiveAccountId();
}

export async function persistActiveAccountId(id: string | null) {
  await weworkSetActiveAccountId(id);
}

/** 连接成功后写入或更新列表项 */
export async function upsertSavedAccount(
  accounts: WeworkSavedAccount[],
  config: WeworkAccountConfig,
  label?: string,
  weworkUserId?: string
): Promise<{ accounts: WeworkSavedAccount[]; account: WeworkSavedAccount }> {
  const fp = configFingerprint(config);
  const now = Date.now();
  const trimmedLabel = label?.trim();
  const trimmedUserId = weworkUserId?.trim();

  let existing: WeworkSavedAccount | undefined;
  if (trimmedUserId) {
    existing = accounts.find((a) => a.weworkUserId === trimmedUserId);
  }
  if (!existing && fp) {
    existing = accounts.find((a) => configFingerprint(a.config) === fp);
  }

  if (existing) {
    const next: WeworkSavedAccount = {
      ...existing,
      config: { ...existing.config, ...config },
      label: trimmedLabel || existing.label,
      weworkUserId: trimmedUserId || existing.weworkUserId,
      lastConnectedAt: now
    };
    await weworkUpsertAccount(accountToDto(next));
    const nextAccounts = accounts.map((a) => (a.id === existing!.id ? next : a));
    return { accounts: nextAccounts, account: next };
  }

  const account: WeworkSavedAccount = {
    id: crypto.randomUUID(),
    label: trimmedLabel || defaultAccountLabel(config) || "WeCom",
    config,
    createdAt: now,
    lastConnectedAt: now,
    weworkUserId: trimmedUserId || undefined
  };
  await weworkUpsertAccount(accountToDto(account));
  return { accounts: [account, ...accounts], account };
}

export async function removeSavedAccount(accounts: WeworkSavedAccount[], id: string) {
  await weworkDeleteAccount(id);
  return accounts.filter((a) => a.id !== id);
}

/** 从通道目录解析登录展示名 */
export function resolveWeworkLoginFromCatalog(channel: ChannelCatalogEntry | null | undefined): {
  label?: string;
  weworkUserId?: string;
} {
  const profile = channel?.login_profile;
  if (!profile?.display_name) {
    return {};
  }
  return {
    label: profile.display_name,
    weworkUserId: profile.user_id || undefined
  };
}

/** 当前已连接账号 */
export async function resolveActiveSavedAccount(): Promise<WeworkSavedAccount | null> {
  const accounts = await loadSavedAccounts();
  if (accounts.length === 0) {
    return null;
  }
  const id = await loadActiveAccountId();
  if (id) {
    const hit = accounts.find((a) => a.id === id);
    if (hit) {
      return hit;
    }
  }
  const sorted = [...accounts].sort((a, b) => (b.lastConnectedAt ?? 0) - (a.lastConnectedAt ?? 0));
  return sorted[0] ?? null;
}
