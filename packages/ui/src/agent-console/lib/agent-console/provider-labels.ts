/** Maps `config.json` `bot_type` id → i18n key under `console.provider_*`. */
const PROVIDER_LABEL_KEYS: Record<string, string> = {
  deepseek: "provider_deepseek",
  openai: "provider_openai",
  chatgpt: "provider_openai",
  chatgptonazure: "provider_azure",
  azure: "provider_azure",
  claude: "provider_claude",
  claudeapi: "provider_claude",
  gemini: "provider_gemini",
  zhipuai: "provider_zhipu",
  moonshot: "provider_moonshot",
  doubao: "provider_doubao",
  dashscope: "provider_dashscope",
  minimax: "provider_minimax",
  linkai: "provider_linkai",
  custom: "provider_custom",
  baidu: "provider_baidu",
  qianfan: "provider_qianfan",
  xunfei: "provider_xunfei",
  modelscope: "provider_modelscope"
};

export function providerLabelKey(botType: string): string {
  const direct = PROVIDER_LABEL_KEYS[botType];
  if (direct) {
    return direct;
  }
  const lower = botType.toLowerCase();
  return PROVIDER_LABEL_KEYS[lower] ?? "provider_unknown";
}
