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
    weworkUserId: dto.weworkUserId,
    contactsSynced: dto.contactsSynced,
    contactsSyncedAt: dto.contactsSyncedAt
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
    weworkUserId: account.weworkUserId,
    contactsSynced: account.contactsSynced,
    contactsSyncedAt: account.contactsSyncedAt
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

function findExistingAccount(
  accounts: WeworkSavedAccount[],
  fingerprint: string,
  weworkUserId?: string
): WeworkSavedAccount | undefined {
  const trimmedUserId = weworkUserId?.trim();
  if (trimmedUserId) {
    const account = accounts.find((item) => item.weworkUserId === trimmedUserId);
    if (account) {
      return account;
    }
  }

  return fingerprint
    ? accounts.find((item) => configFingerprint(item.config) === fingerprint)
    : undefined;
}

function buildUpdatedAccount(params: {
  config: WeworkAccountConfig;
  existing: WeworkSavedAccount;
  label?: string;
  now: number;
  weworkUserId?: string;
}): WeworkSavedAccount {
  const { config, existing, label, now, weworkUserId } = params;
  return {
    ...existing,
    config: { ...existing.config, ...config },
    label: label?.trim() || existing.label,
    weworkUserId: weworkUserId?.trim() || existing.weworkUserId,
    lastConnectedAt: now,
    contactsSynced: existing.contactsSynced,
    contactsSyncedAt: existing.contactsSyncedAt
  };
}

function buildNewAccount(params: {
  config: WeworkAccountConfig;
  label?: string;
  now: number;
  weworkUserId?: string;
}): WeworkSavedAccount {
  const { config, label, now, weworkUserId } = params;
  return {
    id: crypto.randomUUID(),
    label: label?.trim() || defaultAccountLabel(config) || "WeCom",
    config,
    createdAt: now,
    lastConnectedAt: now,
    weworkUserId: weworkUserId?.trim() || undefined,
    contactsSynced: false,
    contactsSyncedAt: undefined
  };
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
  const existing = findExistingAccount(accounts, fp, weworkUserId);

  if (existing) {
    const next = buildUpdatedAccount({ config, existing, label, now, weworkUserId });
    await weworkUpsertAccount(accountToDto(next));
    const nextAccounts = accounts.map((account) => (account.id === existing.id ? next : account));
    return { accounts: nextAccounts, account: next };
  }

  const account = buildNewAccount({ config, label, now, weworkUserId });
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
