import { invokeWrapper } from "./invoke";
import { TauriCmd } from "../enums/tauri-cmd";
import type { ChannelSavedAccountDto } from "../../contracts/contracts";

export type { ChannelAccountConfigDto, ChannelSavedAccountDto } from "../../contracts/contracts";

export async function channelListAccounts(channel: string): Promise<ChannelSavedAccountDto[]> {
  return invokeWrapper<ChannelSavedAccountDto[]>(TauriCmd.ChannelListAccounts, { channel });
}

export async function channelGetActiveAccountId(channel: string): Promise<string | null> {
  return invokeWrapper<string | null>(TauriCmd.ChannelGetActiveAccountId, { channel });
}

export async function channelSetActiveAccountId(channel: string, id: string | null): Promise<void> {
  await invokeWrapper<void>(TauriCmd.ChannelSetActiveAccountId, { channel, id });
}
