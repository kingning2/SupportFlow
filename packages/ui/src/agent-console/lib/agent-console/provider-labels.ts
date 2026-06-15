const PROVIDER_LABELS: Record<string, string> = {
  deepseek: "DeepSeek",
  openai: "OpenAI",
  chatgpt: "OpenAI",
  chatgptonazure: "Azure OpenAI",
  azure: "Azure OpenAI",
  claude: "Claude",
  claudeapi: "Claude",
  gemini: "Gemini",
  zhipuai: "智谱 AI",
  moonshot: "Moonshot",
  doubao: "豆包",
  dashscope: "DashScope",
  minimax: "MiniMax",
  linkai: "LinkAI",
  ollama: "Ollama（本地）",
  custom: "自定义",
  baidu: "百度千帆",
  qianfan: "百度千帆",
  xunfei: "讯飞星火",
  modelscope: "ModelScope"
};

export function providerLabel(botType: string): string {
  const direct = PROVIDER_LABELS[botType];
  if (direct) {
    return direct;
  }
  const lower = botType.toLowerCase();
  return PROVIDER_LABELS[lower] ?? botType;
}
