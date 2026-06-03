import {
  channelFieldValueString,
  isChannelMaskedSecret,
  type ChannelCatalogEntry
} from "@supportflow/shared/tauri-bridge/cmd/channel-python-channels";

const SEGMENTED_TAB_BASE = "flex-1 rounded-md px-3 py-1.5 text-xs font-medium";
const SEGMENTED_TAB_ACTIVE =
  "bg-white text-slate-800 shadow-sm dark:bg-slate-700 dark:text-slate-100";
const SEGMENTED_TAB_INACTIVE =
  "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200";

/** 判断企微机器人是否已经填写了可用凭证，屏蔽后的密钥不算可连接凭证。 */
export function wecomHasCreds(channel: ChannelCatalogEntry): boolean {
  const botId = channel.fields.find((field) => field.key === "wecom_bot_id");
  const secret = channel.fields.find((field) => field.key === "wecom_bot_secret");
  const secretValue = channelFieldValueString(secret?.value);
  return !!(
    channelFieldValueString(botId?.value) &&
    secretValue &&
    !isChannelMaskedSecret(secretValue)
  );
}

/** 生成扫码/手动模式分段按钮样式，保持不同渠道面板一致。 */
export function channelModeTabClass(active: boolean): string {
  return `${SEGMENTED_TAB_BASE} ${active ? SEGMENTED_TAB_ACTIVE : SEGMENTED_TAB_INACTIVE}`;
}
