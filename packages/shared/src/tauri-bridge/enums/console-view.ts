/** SupportFlow console sidebar views (mirrors `data-view` in chat.html). */
export enum ConsoleView {
  Chat = "chat",
  Config = "config",
  Models = "models",
  Skills = "skills",
  Memory = "memory",
  Knowledge = "knowledge",
  Channels = "channels",
  Tasks = "tasks",
  Logs = "logs"
}

export const CONSOLE_VIEW_VALUES = Object.values(ConsoleView) as ConsoleView[];

export function isConsoleView(value: string): value is ConsoleView {
  return (CONSOLE_VIEW_VALUES as string[]).includes(value);
}
