/** 主窗内容区 className（各 flavor 通过 AppShellLayout 选用） */
export const APP_SHELL_CONTENT_CLASS = {
  /** 完整控制台：保留 p-3，页面可用 -m-3 铺满 */
  console: "relative flex min-h-0 flex-1 flex-col overflow-hidden bg-white p-3",
  /** 单通道 flavor：无外边距 */
  channel: "relative flex min-h-0 flex-1 flex-col overflow-hidden bg-white dark:bg-[#111]"
} as const;
