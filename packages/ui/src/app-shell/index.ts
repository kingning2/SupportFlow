export { AppShellLayout, type AppShellLayoutProps } from "./app-shell-layout";
export { APP_SHELL_CONTENT_CLASS } from "./shell-content-class";
export { DesktopAppLayout, type DesktopAppLayoutProps } from "./desktop-app-layout";
export { DesktopAppRoot } from "./desktop-app-root";
export { MainWindowBgGuard } from "./main-window-bg-guard";

/** @deprecated 使用 `DesktopAppRoot` */
export { DesktopAppRoot as ChannelAppRoot } from "./desktop-app-root";
/** @deprecated 使用 `AppShellLayout` */
export { AppShellLayout as ChannelShellLayout } from "./app-shell-layout";
/** @deprecated 使用 `TitleBarAccent` from `@supportflow/ui/title-bar` */
export type { TitleBarAccent as ChannelShellAccent } from "@supportflow/ui/title-bar";

export {
  channelAction,
  fetchChannelConsoleApi,
  channelLangFromI18n,
  fetchChannels,
  type ChannelActionRequest
} from "./channel-bridge";
