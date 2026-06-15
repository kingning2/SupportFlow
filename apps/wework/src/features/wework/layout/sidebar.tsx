"use client";

import { IconApartment } from "@douyinfe/semi-icons";
import { Avatar, Layout, Nav } from "@douyinfe/semi-ui-19";
import type { OnSelectedData } from "@douyinfe/semi-ui-19/lib/es/navigation";
import { useEffect, useMemo, useState } from "react";

import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

import { AccountAvatar } from "../accounts/avatar";
import { WEWORK_NAV_GROUPS } from "../constants/wework-nav";
import type { WeworkConnectionStatus } from "../types/wework-conversation";

const { Sider } = Layout;

const SIDER_WIDTH = 220;

export interface SidebarProps {
  activeRoute: WeworkConsoleRoute;
  onNavigate: (route: WeworkConsoleRoute) => void;
  connectionStatus: WeworkConnectionStatus;
  connectedAccountName?: string | null;
  openGroups: Record<string, boolean>;
  onToggleGroup: (groupId: string) => void;
}

function connectionLabel(status: WeworkConnectionStatus): string {
  switch (status) {
    case "ready":
      return "已连接";
    case "connecting":
      return "连接中";
    default:
      return "未连接";
  }
}

export function Sidebar({
  activeRoute,
  onNavigate,
  connectionStatus,
  connectedAccountName,
  openGroups
}: SidebarProps) {
  const showAccount = connectionStatus === "ready" && connectedAccountName;

  const navItems = useMemo(
    () =>
      WEWORK_NAV_GROUPS.map((group) => ({
        itemKey: group.id,
        text: group.label,
        items: group.items.map((item) => ({
          itemKey: item.route,
          text: item.label,
          icon: item.icon
        }))
      })),
    []
  );

  const [openKeys, setOpenKeys] = useState<string[]>(() =>
    WEWORK_NAV_GROUPS.filter((group) => openGroups[group.id]).map((group) => group.id)
  );

  useEffect(() => {
    setOpenKeys(WEWORK_NAV_GROUPS.filter((group) => openGroups[group.id]).map((group) => group.id));
  }, [openGroups]);

  const handleSelect = (data: OnSelectedData) => {
    const key = String(data.itemKey ?? "");
    if (Object.values(WeworkConsoleRoute).includes(key as WeworkConsoleRoute)) {
      onNavigate(key as WeworkConsoleRoute);
    }
  };

  const header = showAccount
    ? {
        logo: <AccountAvatar name={connectedAccountName} size="sm" />,
        text: connectedAccountName
      }
    : {
        logo: (
          <Avatar
            size="small"
            className="wework-sider-logo"
            style={{
              background: "linear-gradient(145deg, #3370ff 0%, #245bdb 100%)",
              color: "#fff"
            }}
          >
            <IconApartment />
          </Avatar>
        ),
        text: "企微智能客服"
      };

  return (
    <Sider className="wework-semi-sider" style={{ width: SIDER_WIDTH, flexShrink: 0 }}>
      <Nav
        className="wework-semi-nav"
        mode="vertical"
        style={{ width: SIDER_WIDTH, height: "100%" }}
        items={navItems}
        selectedKeys={[activeRoute]}
        openKeys={openKeys}
        onOpenChange={(data) => setOpenKeys((data.openKeys ?? []).map(String))}
        onSelect={handleSelect}
        header={header}
        footer={{
          collapseButton: true,
          collapseText: () => connectionLabel(connectionStatus)
        }}
      />
    </Sider>
  );
}
