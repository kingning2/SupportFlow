export enum LocalCacheKey {
  /** SupportFlow web console session id */
  AgentSessionId = "agent_session_id",
  ConsoleTheme = "console_theme",
  ConsoleLang = "console_lang",
  /** 企微 Inbox 当前选中的 conversation_id */
  WeworkActiveConversationId = "wework_active_conversation_id",
  /** 企微壳上次打开的导航项 */
  WeworkConsoleRoute = "wework_console_route",
  /** 企微已保存的账号配置列表（JSON） */
  WeworkSavedAccounts = "wework_saved_accounts",
  /** 当前已连接的企微账号 id（对应 WeworkSavedAccounts 项） */
  WeworkActiveAccountId = "wework_active_account_id"
}
