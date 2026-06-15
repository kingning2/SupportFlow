/** 企微独立 Webview 主壳导航（左侧 Tab）。 */
export enum WeworkConsoleRoute {
  Inbox = "inbox",
  Knowledge = "knowledge",
  Skills = "skills",
  AiChat = "ai_chat",
  Mcp = "mcp",
  AiConfig = "ai_config",
  Account = "account"
}

export const WEWORK_CONSOLE_ROUTE_VALUES = Object.values(
  WeworkConsoleRoute
) as WeworkConsoleRoute[];

export function isWeworkConsoleRoute(value: string): value is WeworkConsoleRoute {
  return (WEWORK_CONSOLE_ROUTE_VALUES as string[]).includes(value);
}
