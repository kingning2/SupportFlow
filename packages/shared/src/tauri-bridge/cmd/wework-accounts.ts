import { invokeWrapper } from "./invoke";
import { TauriCmd } from "../enums/tauri-cmd";

export interface WeworkAccountConfigDto {
  weworkExePath?: string;
  weworkVersion?: string;
  weworkSmart?: boolean;
  weworkInitWaitSeconds?: number;
}

export interface WeworkSavedAccountDto {
  id: string;
  label: string;
  config: WeworkAccountConfigDto;
  createdAt: number;
  lastConnectedAt?: number;
  weworkUserId?: string;
  contactsSynced: boolean;
  contactsSyncedAt?: number;
}

export async function weworkListAccounts(): Promise<WeworkSavedAccountDto[]> {
  return invokeWrapper<WeworkSavedAccountDto[]>(TauriCmd.WeworkListAccounts);
}

export async function weworkUpsertAccount(
  account: WeworkSavedAccountDto
): Promise<WeworkSavedAccountDto> {
  return invokeWrapper<WeworkSavedAccountDto>(TauriCmd.WeworkUpsertAccount, { account });
}

export async function weworkDeleteAccount(id: string): Promise<void> {
  await invokeWrapper<void>(TauriCmd.WeworkDeleteAccount, { id });
}

export async function weworkGetActiveAccountId(): Promise<string | null> {
  return invokeWrapper<string | null>(TauriCmd.WeworkGetActiveAccountId);
}

export async function weworkSetActiveAccountId(id: string | null): Promise<void> {
  await invokeWrapper<void>(TauriCmd.WeworkSetActiveAccountId, { id });
}

export async function weworkMarkContactsSynced(
  weworkUserId: string,
  syncedAt: number
): Promise<void> {
  await invokeWrapper<void>(TauriCmd.WeworkMarkContactsSynced, {
    weworkUserId,
    syncedAt
  });
}

export async function weworkContactsSynced(weworkUserId: string): Promise<boolean> {
  return invokeWrapper<boolean>(TauriCmd.WeworkContactsSynced, { weworkUserId });
}
