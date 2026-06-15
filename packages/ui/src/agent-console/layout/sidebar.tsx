"use client";

import { useEffect, useMemo, useState } from "react";
import { Avatar, Layout, Nav, Space, Typography } from "@douyinfe/semi-ui-19";
import { IconSemiLogo } from "@douyinfe/semi-icons";

import { cn } from "@supportflow/shared";
import type { ConsoleView } from "@supportflow/shared/tauri-bridge/enums";

import type { SidebarGroupId, SidebarNavGroup } from "../constants/sidebar-nav";

const { Sider } = Layout;
const { Text } = Typography;

const SIDER_WIDTH = 208;

interface SidebarProps {
  navGroups: SidebarNavGroup[];
  activeView: ConsoleView;
  onNavigate: (view: ConsoleView) => void;
  openGroups: Record<SidebarGroupId, boolean>;
  onToggleGroup: (groupId: SidebarGroupId) => void;
  mobileOpen: boolean;
  onCloseMobile: () => void;
}

export function Sidebar({
  navGroups,
  activeView,
  onNavigate,
  openGroups,
  onToggleGroup,
  mobileOpen,
  onCloseMobile
}: SidebarProps) {
  const [openKeys, setOpenKeys] = useState<string[]>(() =>
    navGroups.filter((group) => openGroups[group.id]).map((group) => group.id)
  );

  useEffect(() => {
    setOpenKeys(navGroups.filter((group) => openGroups[group.id]).map((group) => group.id));
  }, [navGroups, openGroups]);

  const navItems = useMemo(
    () =>
      navGroups.map((group) => ({
        itemKey: group.id,
        text: group.label,
        items: group.items.map((item) => ({
          itemKey: item.view,
          text: item.label,
          icon: item.icon
        }))
      })),
    [navGroups]
  );

  return (
    <>
      <Sider
        className={cn(
          "agent-console-sider",
          mobileOpen ? "agent-console-sider--open" : "agent-console-sider--closed"
        )}
        style={{
          width: SIDER_WIDTH,
          flexShrink: 0,
          background: "var(--console-sidebar-bg, #1f2329)",
          color: "var(--semi-color-text-2)"
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            height: 56,
            padding: "0 20px",
            borderBottom: "1px solid rgb(255 255 255 / 0.08)"
          }}
        >
          <Avatar size="small" style={{ background: "var(--semi-color-primary)", color: "#fff" }}>
            <IconSemiLogo />
          </Avatar>
          <Space vertical spacing={0}>
            <Text strong style={{ color: "#fff" }}>
              SupportFlow
            </Text>
            <Text size="small" type="tertiary">
              控制台
            </Text>
          </Space>
        </div>

        <Nav
          mode="vertical"
          style={{ height: "calc(100% - 56px - 48px)", background: "transparent", borderRight: 0 }}
          items={navItems}
          selectedKeys={[activeView]}
          openKeys={openKeys}
          onOpenChange={(data) => {
            const keys = (data.openKeys ?? []).map(String);
            setOpenKeys(keys);
            navGroups.forEach((group) => {
              const shouldOpen = keys.includes(group.id);
              if (shouldOpen !== openGroups[group.id]) {
                onToggleGroup(group.id);
              }
            });
          }}
          onSelect={(data) => {
            const view = String(data.itemKey ?? "") as ConsoleView;
            onNavigate(view);
            onCloseMobile();
          }}
        />

        <div style={{ padding: "12px 16px", borderTop: "1px solid rgb(255 255 255 / 0.08)" }}>
          <Space spacing="tight">
            <span
              style={{
                width: 6,
                height: 6,
                borderRadius: "50%",
                background: "var(--semi-color-success)"
              }}
            />
            <Text size="small" type="tertiary">
              SupportFlow Desktop
            </Text>
          </Space>
        </div>
      </Sider>

      {mobileOpen ? (
        <div
          role="presentation"
          className="agent-console-sider-backdrop"
          onClick={onCloseMobile}
          onKeyDown={() => undefined}
        />
      ) : null}
    </>
  );
}
