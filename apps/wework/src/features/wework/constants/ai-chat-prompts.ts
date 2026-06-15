import type { ExamplePrompt } from "@supportflow/ui/agent-console/constants/example-prompts";

export const WEWORK_USER_CHAT_PROMPTS: ExamplePrompt[] = [
  {
    id: "whoami",
    title: "当前账号",
    text: "我现在用的是哪个企微账号？",
    prompt: "请根据当前上下文，说明我现在登录/使用的是哪个企业微信账号（名称、备注、连接状态）。",
    iconBg: "var(--semi-color-primary-light-default)",
    iconColor: "var(--semi-color-primary)"
  },
  {
    id: "status",
    title: "连接状态",
    text: "我的企微通道现在是什么状态？",
    prompt: "请说明当前企业微信通道的连接状态，以及若未就绪需要我做什么。",
    iconBg: "var(--semi-color-info-light-default)",
    iconColor: "var(--semi-color-info)"
  },
  {
    id: "profile",
    title: "账号信息",
    text: "汇总一下我当前账号的关键配置。",
    prompt:
      "请汇总我当前企微账号的关键信息（显示名称、DLL 路径、智能等待等已保存配置），不要查询无关业务数据。",
    iconBg: "var(--semi-color-success-light-default)",
    iconColor: "var(--semi-color-success)"
  },
  {
    id: "model",
    title: "当前模型",
    text: "现在对话用的是哪个 AI 模型？",
    prompt: "请说明当前 Agent 使用的 bot_type、模型名称，以及是否为本机 Ollama。",
    iconBg: "var(--semi-color-warning-light-default)",
    iconColor: "var(--semi-color-warning)"
  }
];
