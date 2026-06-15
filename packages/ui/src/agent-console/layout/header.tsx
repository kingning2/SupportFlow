"use client";

import { Breadcrumb, IconButton, Layout, Space } from "@douyinfe/semi-ui-19";
import { IconExternalOpen, IconHistory, IconMenu, IconMoon, IconSun } from "@douyinfe/semi-icons";

import { CONSOLE_BRAND, getBreadcrumbLabels } from "../constants/sidebar-nav";
import type { ConsoleView, ChannelCatalogEntryId } from "@supportflow/shared/tauri-bridge/enums";
import type { ConsoleTheme } from "../lib/agent-console/theme-sync";

const { Header: SemiHeader } = Layout;

interface ConsoleHeaderProps {
  activeView: ConsoleView;
  devChannel: ChannelCatalogEntryId | null;
  theme: ConsoleTheme;
  onToggleTheme: () => void;
  onToggleSessions: () => void;
  onToggleMobileSidebar: () => void;
}

export function ConsoleHeader({
  activeView,
  devChannel,
  theme,
  onToggleTheme,
  onToggleSessions,
  onToggleMobileSidebar
}: ConsoleHeaderProps) {
  const { groupLabel, pageLabel } = getBreadcrumbLabels(activeView, devChannel);

  return (
    <SemiHeader
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        height: 56,
        lineHeight: "inherit",
        padding: "0 16px",
        borderBottom: "1px solid var(--semi-color-border)",
        background: "var(--semi-color-bg-0)"
      }}
    >
      <IconButton
        className="agent-console-header-menu"
        icon={<IconMenu />}
        aria-label="打开导航"
        theme="borderless"
        type="tertiary"
        onClick={onToggleMobileSidebar}
      />

      <IconButton
        icon={<IconHistory />}
        aria-label="历史会话"
        theme="borderless"
        type="tertiary"
        onClick={onToggleSessions}
      />

      <Breadcrumb className="agent-console-breadcrumb">
        <Breadcrumb.Item>{groupLabel}</Breadcrumb.Item>
        <Breadcrumb.Item>{pageLabel}</Breadcrumb.Item>
      </Breadcrumb>

      <div style={{ flex: 1 }} />

      <Space>
        <IconButton
          icon={theme === "dark" ? <IconSun /> : <IconMoon />}
          aria-label="切换主题"
          theme="borderless"
          type="tertiary"
          onClick={onToggleTheme}
        />
        <IconButton
          icon={<IconExternalOpen />}
          aria-label="GitHub"
          theme="borderless"
          type="tertiary"
          onClick={() => window.open(CONSOLE_BRAND.githubUrl, "_blank", "noopener,noreferrer")}
        />
      </Space>
    </SemiHeader>
  );
}
