"use client";

import { Avatar, Dropdown, IconButton, Space, Tag, Typography } from "@douyinfe/semi-ui-19";
import {
  IconClose,
  IconCopy,
  IconHelpCircleStroked,
  IconInfoCircle,
  IconKey,
  IconMailStroked1,
  IconMaximize2Stroked,
  IconMenu,
  IconMinusStroked,
  IconSync
} from "@douyinfe/semi-icons";
import { memo, useMemo, useState } from "react";

import { mainWindow } from "@supportflow/shared/tauri-bridge/window/main-window";
import { formatLicenseInvalidReason } from "@supportflow/shared/tauri-bridge/license-error";
import { useOptionalLicenseGate } from "@supportflow/ui/license";

import { LicenseActivationModal, LicenseMachineCodeModal } from "./license-modals";

const { Title } = Typography;

export type TitleBarAccent = {
  logoGradient: string;
  title: string;
  barClassName: string;
  logoText: string;
  titleClassName?: string;
  controlClassName?: string;
};

const INTERACTIVE_TITLE_BAR_SELECTOR =
  "button, a, input, select, textarea, [role='menuitem'], .semi-dropdown";

type MoreMenuItem = {
  id: string;
  label: string;
  icon: React.ReactNode;
};

const MORE_MENU_ITEMS: MoreMenuItem[] = [
  { id: "feedback", label: "反馈", icon: <IconMailStroked1 /> },
  { id: "contact_support", label: "联系支持", icon: <IconHelpCircleStroked /> },
  { id: "online_help", label: "在线帮助", icon: <IconHelpCircleStroked /> },
  { id: "check_updates", label: "检查更新", icon: <IconSync /> },
  { id: "about", label: "关于", icon: <IconInfoCircle /> }
];

function handleTitleBarMouseDown(e: React.MouseEvent<HTMLDivElement>) {
  if (e.buttons !== 1) return;
  const target = e.target as HTMLElement;
  if (target.closest(INTERACTIVE_TITLE_BAR_SELECTOR)) return;
  void mainWindow.startDragging();
}

const TitleBar = memo((props: { height?: number; accent?: TitleBarAccent }) => {
  const h = props.height ?? 40;
  const accent = props.accent;
  const title = accent?.title ?? "SupportFlow";
  const logoText = accent?.logoText ?? "T";

  const [activationOpen, setActivationOpen] = useState(false);
  const [machineCodeOpen, setMachineCodeOpen] = useState(false);
  const licenseGate = useOptionalLicenseGate();
  const licenseWarning =
    licenseGate && !licenseGate.loading && !licenseGate.valid
      ? formatLicenseInvalidReason(licenseGate.reasonLabel)
      : null;

  const menu = useMemo(
    () => (
      <Dropdown.Menu>
        <Dropdown.Item icon={<IconKey />} onClick={() => setActivationOpen(true)}>
          订阅激活
        </Dropdown.Item>
        <Dropdown.Item icon={<IconCopy />} onClick={() => setMachineCodeOpen(true)}>
          机器码
        </Dropdown.Item>
        <Dropdown.Divider />
        {MORE_MENU_ITEMS.map(({ id, label, icon }) => (
          <Dropdown.Item key={id} icon={icon} disabled>
            {label}
          </Dropdown.Item>
        ))}
      </Dropdown.Menu>
    ),
    []
  );

  return (
    <div
      role="banner"
      data-tauri-drag-region
      className={accent?.barClassName ?? "bg-card/90 backdrop-blur"}
      style={{
        display: "flex",
        width: "100%",
        height: h,
        cursor: "default",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "0 12px",
        userSelect: "none"
      }}
      onMouseDown={handleTitleBarMouseDown}
    >
      <Space style={{ minWidth: 0, flex: 1 }}>
        <Avatar
          size="small"
          style={{
            background: accent ? undefined : "linear-gradient(145deg, #2b7fff 0%, #155dfc 100%)",
            color: "#fff",
            fontWeight: 700
          }}
          className={accent?.logoGradient}
        >
          {logoText}
        </Avatar>
        <Title
          heading={6}
          ellipsis
          style={{ margin: 0, maxWidth: "100%" }}
          className={accent?.titleClassName}
        >
          {title}
        </Title>
        {licenseWarning ? (
          <Tag color="red" size="small">
            {licenseWarning}
          </Tag>
        ) : null}
      </Space>

      <div onMouseDown={(e: React.MouseEvent) => e.stopPropagation()}>
        <Space>
          <Dropdown trigger="click" position="bottomRight" render={menu}>
            <IconButton icon={<IconMenu />} aria-label="菜单" theme="borderless" type="tertiary" />
          </Dropdown>

          <LicenseActivationModal open={activationOpen} onOpenChange={setActivationOpen} />
          <LicenseMachineCodeModal open={machineCodeOpen} onOpenChange={setMachineCodeOpen} />

          <IconButton
            icon={<IconMinusStroked />}
            aria-label="最小化"
            theme="borderless"
            type="tertiary"
            onClick={() => void mainWindow.minimize()}
          />
          <IconButton
            icon={<IconMaximize2Stroked />}
            aria-label="最大化"
            theme="borderless"
            type="tertiary"
            onClick={() => void mainWindow.toggleMaximize()}
          />
          <IconButton
            icon={<IconClose />}
            aria-label="关闭"
            theme="borderless"
            type="danger"
            onClick={() => void mainWindow.close()}
          />
        </Space>
      </div>
    </div>
  );
});

TitleBar.displayName = "TitleBar";

export default TitleBar;
